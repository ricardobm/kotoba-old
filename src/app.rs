use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::RwLock;

use slog::Logger;

use audio;
use japanese;
use japanese::db;
use logging;
use util::{Cache, CacheKey, CacheMap, CacheVal};
use wiki::{WikiController};

/// Name of the root data directory. Used when looking up the data directory.
const DATA_DIR: &str = "kotoba-data";

/// If `true` will import the dictionary data from zip files.
const FROM_ZIP: bool = false;

/// Maintains the global application state and provides top level run methods.
pub struct App {
	pub log: Logger,

	japanese_audio: audio::AudioLoader<japanese::JapaneseAudioQuery>,

	database:  Arc<db::Root>,
	ring_log:  logging::RingLogger,
	dict:      japanese::Dictionary,
	cache_map: CacheMap,

	wiki: Arc<RwLock<WikiController>>,

	_compat_log_guard: slog_scope::GlobalLoggerGuard,
}

#[allow(dead_code)]
impl App {
	/// Initializes the application state and returns the static [App] instance.
	pub fn get() -> &'static App {
		lazy_static! {
			static ref APP: App = {
				use slog::Drain;

				// Compatibility with libraries using `log`

				// Logging schema
				// ==============
				//
				//     ┌─────┐
				//     │     │  ← ← ←  [filter > info]     ┌───────────────────┐
				//     │  T  │                ↑            │                   │
				//     │  E  │              [dup]        ← │ log compatibility │
				//     │  R  │                ↓            │                   │
				//     │  M  │        ┌───────────────┐    └───────────────────┘
				//     │  I  │        │ App::ring_log │
				//     │  N  │        └───────────────┘
				//     │  A  │                ↑            ┌───────────────────┐
				//     │  L  │        ┌───────────────┐    │                   │
				//     │     │  ← ← ← │   App::log    │  ← │ application logs  │
				//     └─────┘        └───────────────┘    │                   │
				//                            ↑            └───────────────────┘
				//                            ↑
				//                            ↑            ┌───────────────────┐
				//  ┌───────────┐     ┌───────────────┐    │                   │
				//  │ Log Cache │ ← ← │ RequestLogger │  ← │   request logs    │
				//  └───────────┘     └───────────────┘    │                   │
				//                                         └───────────────────┘
				//

				// Root drain that outputs to the terminal
				let term = slog_term::term_compact();
				let term = std::sync::Mutex::new(term);

				// The root logger, outputting to the terminal
				let term = Logger::root(term.fuse(), o!());

				// Ring drain that keep all entries for `/api/logs`
				let ring_log = logging::RingLogger::new(1000);

				// Filter out debug/trace entries from libraries
				let filter = slog::LevelFilter::new(term.clone(), slog::Level::Info);

				// The compatibility logs go filtered to `term` and unfiltered
				// to the ring logger.
				let compat_log = slog::Duplicate::new(ring_log.clone(), filter);

				// Setup the compatibility logger
				let compat_log = Logger::root(compat_log.fuse(), o!("library" => true));
				let compat_log_guard = slog_scope::set_global_logger(compat_log);
				slog_stdlog::init().unwrap();

				// Application logs go to the ring logger and `term`.
				let app_log = slog::Duplicate::new(ring_log.clone(), term);
				let app_log = Logger::root(app_log.fuse(), o!());

				time!(t_init);
				info!(app_log, "starting application");

				info!(
					app_log,
					"data directory is {path}",
					path = App::data_dir().to_string_lossy().as_ref()
				);

				let audio_cache_dir = App::data_dir().join("audio");

				let wiki = WikiController::new(App::data_dir().join("wiki"));

				let db = App::load_db(&app_log.new(o!("op" => "database loading")));
				let db = Arc::new(db);
				let app = App {
					log:      app_log,
					ring_log: ring_log,
					database: db.clone(),
					dict:     japanese::Dictionary::new(db.clone()),
					cache_map: CacheMap::new(),

					wiki: Arc::new(RwLock::new(wiki)),

					japanese_audio: japanese::new_audio_loader(&audio_cache_dir),

					_compat_log_guard: compat_log_guard,
				};

				trace!(app.log, "application initialized"; t_init);

				app
			};
		}
		&APP
	}

	/// Creates a new [Logger] for a request.
	///
	/// A request logger will still log entries globally, but will also store
	/// entries in the [RequestLogStore.]
	///
	/// Returns the created logger
	pub fn request_log<T>(&self, values: slog::OwnedKV<T>) -> (Logger, logging::RequestLogStore)
	where
		T: slog::SendSyncRefUnwindSafeKV + 'static,
	{
		let logger = logging::RequestLogger::new(self.log.clone());
		let store = logger.store();
		(Logger::root(logger, values), store)
	}

	pub fn japanese_audio(&self) -> &audio::AudioLoader<japanese::JapaneseAudioQuery> {
		&self.japanese_audio
	}

	/// Return the latest log entries for the application.
	pub fn all_logs(&self) -> Vec<logging::LogEntry> {
		self.ring_log.entries()
	}

	/// The static [db::Root] instance.
	pub fn db(&self) -> Arc<db::Root> {
		self.database.clone()
	}

	pub fn dictionary(&self) -> &japanese::Dictionary {
		&self.dict
	}

	/// Returns a global cache instance for a given key and value types.
	pub fn cache<K: CacheKey + 'static, V: CacheVal + 'static>(&self) -> Cache<K, V> {
		self.cache_map.get()
	}

	/// Returns the global [WikiController] instance.
	pub fn wiki(&self) -> Arc<RwLock<WikiController>> {
		self.wiki.clone()
	}

	/// Root directory that contains static application data.
	pub fn data_dir() -> &'static Path {
		lazy_static! {
			static ref DATA_PATH: PathBuf = {
				// Find the data directory starting at the current directory
				// and moving up.
				let mut cur_dir = std::env::current_dir().unwrap();
				let data_path = loop {
					let mut cur_path = PathBuf::from(&cur_dir);
					cur_path.push(DATA_DIR);

					let mut test_path = PathBuf::from(&cur_path);
					test_path.push("import");
					if let Ok(md) = std::fs::metadata(&test_path) {
						if md.is_dir() {
							break Some(cur_path);
						}
					}

					if let Some(dir) = cur_dir.parent() {
						cur_dir = dir.to_path_buf();
					} else {
						break None;
					}
				};

				let data_path = data_path.expect("could not find the user directory");
				data_path
			};
		}
		DATA_PATH.as_path()
	}

	fn load_db(log: &Logger) -> db::Root {
		// Figure out the imported dictionary path
		let data_dir = Self::data_dir();
		let mut dict_path = PathBuf::from(data_dir);
		dict_path.push("dict");
		dict_path.push("imported.db");
		let dict_dir = dict_path.to_string_lossy();

		// Attempt to load the database from the imported path
		time!(t_load);
		let mut db = if let Some(db) = db::Root::load(log, &dict_path).unwrap() {
			info!(log, "loaded from {}", dict_dir; t_load);
			db
		} else {
			// If the database could not be loaded, attempt to import the entries
			let import_path = {
				let mut p = PathBuf::from(data_dir);
				p.push(if FROM_ZIP { "import" } else { "import-files" });
				p
			};
			let import_dir = import_path.to_string_lossy();
			warn!(log, "not found in {}, importing from {}", dict_dir, import_dir);

			let mut db = db::Root::new();
			japanese::import::from_yomichan(&mut db, import_path).unwrap();
			info!(log, "imported{} files", if FROM_ZIP { " zip" } else { "" }; t_load);

			time!(t_update);
			let (kanji_count, terms_count) = db.update_frequency();
			info!(
				log,
				"updated frequency for {} kanji and {} terms in {}", kanji_count, terms_count, t_update
			);

			db.merge_entries(log);

			db.update_index(log, true);
			db.save(log, &dict_path).unwrap();
			db
		};

		// Updates the index file in case it is missing
		if db.update_index(log, false) {
			db.save(log, &dict_path).unwrap();
		}

		db
	}
}

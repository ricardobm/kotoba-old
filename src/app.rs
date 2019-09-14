use std::path::{Path, PathBuf};
use std::time;

use slog::*;

use db;
use japanese;
use pronunciation;

/// Name of the root data directory. Used when looking up the data directory.
const DATA_DIR: &str = "hongo-data";

/// If `true` will import the dictionary data from zip files.
const FROM_ZIP: bool = false;

/// Maintains the global application state and provides top level run methods.
pub struct App {
	pub log: Logger,

	dict:     japanese::Dictionary,
	audio_ja: pronunciation::JapaneseService,

	_global_log_guard: slog_scope::GlobalLoggerGuard,
}

#[allow(dead_code)]
impl App {
	/// Initializes the application state and returns the static [App] instance.
	pub fn get() -> &'static App {
		lazy_static! {
			static ref APP: App = {
				let term = slog_term::term_compact();
				let term = std::sync::Mutex::new(term).fuse();

				let global_log = slog::Logger::root(term, o!());
				let root_log = global_log.clone();

				let global_log_guard = slog_scope::set_global_logger(global_log);
				slog_stdlog::init().unwrap();

				time!(t_init);
				info!(root_log, #"app", "starting application");

				let audio_ja = pronunciation::JapaneseService::new(&App::data_dir().join("audio"));
				let app = App {
					log:      root_log,
					dict:     japanese::Dictionary::new(App::load_db()),
					audio_ja: audio_ja,

					_global_log_guard: global_log_guard,
				};

				trace!(app.log, #"app", "application initialized"; t_init);

				app
			};
		}
		&APP
	}

	/// The static [db::Root] instance.
	pub fn db(&self) -> &db::Root {
		self.dict.get_db()
	}

	pub fn dictionary(&self) -> &japanese::Dictionary {
		&self.dict
	}

	pub fn pronunciation(&self) -> &pronunciation::JapaneseService {
		&self.audio_ja
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
				println!("Found data directory at {}", data_path.to_string_lossy());
				data_path
			};
		}
		DATA_PATH.as_path()
	}

	fn load_db() -> db::Root {
		// Figure out the imported dictionary path
		let data_dir = Self::data_dir();
		let mut dict_path = PathBuf::from(data_dir);
		dict_path.push("dict");
		dict_path.push("imported.db");
		let dict_path_str = dict_path.to_string_lossy();

		// Attempt to load the database from the imported path
		let start = time::Instant::now();
		let mut db = if let Some(db) = db::Root::load(&dict_path).unwrap() {
			println!(
				"Loaded database from {} in {:.3}s",
				dict_path_str,
				start.elapsed().as_secs_f64()
			);
			db
		} else {
			// If the database could not be loaded, attempt to import the entries
			println!("Database not found in {}!", dict_path_str);

			let import_path = {
				let mut p = PathBuf::from(data_dir);
				p.push(if FROM_ZIP { "import" } else { "import-files" });
				p
			};
			println!("Importing from {}...", import_path.to_string_lossy());

			let mut db = db::Root::new();
			crate::import::from_yomichan(&mut db, import_path).unwrap();
			println!(
				"... imported{} files in {:.3}s",
				if FROM_ZIP { " zip" } else { "" },
				start.elapsed().as_secs_f64()
			);

			let start = time::Instant::now();
			let (kanji_count, terms_count) = db.update_frequency();
			println!(
				"... updated frequency metadata for {} kanji and {} terms in {:.3}s",
				kanji_count,
				terms_count,
				start.elapsed().as_secs_f64()
			);

			db.merge_entries();

			db.update_index(true);
			db.save(&dict_path).unwrap();
			db
		};

		// Updates the index file in case it is missing
		if db.update_index(false) {
			db.save(&dict_path).unwrap();
		}

		db
	}
}

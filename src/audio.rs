//! Support for audio pronunciation loading.

use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use slog::Logger;

use itertools::*;
use regex::Regex;

use util;

/// Completed result for an [AudioQuery].
pub struct AudioResult {
	_items:   Vec<AudioResultItem>,
	_sources: Vec<AudioSourceInfo>,
}

impl AudioResult {
	/// Iterate over each [AudioResultItem] in the results with a closure.
	///
	/// The closure can return `false` to stop the iteration.
	///
	/// Returns `false` if the iteration was stopped by the closure.
	pub fn each<F>(&self, _f: F) -> bool
	where
		F: FnMut(&AudioResultItem) -> bool,
	{
		panic!()
	}

	/// Iterate over each [AudioSourceInfo] in the results with a closure.
	///
	/// The closure can return `false` to stop the iteration.
	///
	/// Returns `false` if the iteration was stopped by the closure.
	pub fn each_source<F>(&self, _f: F) -> bool
	where
		F: FnMut(&AudioSourceInfo) -> bool,
	{
		panic!()
	}
}

/// Singe item in an [AudioResult].
///
/// This is either the complete audio data or an error.
pub type AudioResultItem = util::Result<AudioInfo>;

/// Single item returned by an [AudioSource]
#[allow(dead_code)]
pub type AudioResultData = util::Result<AudioData>;

#[derive(Clone)]
pub struct AudioInfo {
	/// Identifier for the audio source that generated this entry.
	pub source: &'static str,

	/// SHA-256 hash for this file.
	pub hash: String,

	/// Relative path for this file, relative to the [AudioLoader] base path.
	///
	/// This always uses `/` as a path separator.
	pub file: String,

	/// Is `true` if this entry was loaded from the cache.
	pub cached: bool,
}

pub struct AudioSourceInfo {
	/// Identifier for this audio source.
	pub id: &'static str,

	/// Human readable name for this audio source.
	pub name: &'static str,

	/// Total results from this source in the [AudioResult].
	pub total_results: usize,

	/// Number of errors generated from this source in the [AudioResult].
	pub total_errors: usize,

	/// Number of seconds elapsed loading this source.
	pub elapsed: f64,
}

/// Audio data and its SHA-256 hash.
pub struct AudioData(pub Vec<u8>, pub String);

/// Trait for a type that can be used in an audio query.
pub trait AudioQuery: Clone + Sync + Send + Hash + Eq + 'static {
	/// SHA-256 hash for this query parameters.
	///
	/// This is used for determining the audio cached locations and to
	/// de-duplicate the queries.
	fn query_hash(&self) -> String;

	/// A string with query information for logging purposes.
	fn query_info(&self) -> String;
}

/// Trait for a source that can be used to load audio data.
pub trait AudioSource<Q: AudioQuery>: Send + Sync {
	/// Returns a copy of this source to be passed to a thread.
	fn copy(&self) -> Box<dyn AudioSource<Q>>;

	/// Short identifier for the source. Must be a valid path component.
	fn id(&self) -> &'static str;

	/// Human readable name for the source.
	fn name(&self) -> &'static str;

	/// Loads a query from this source.
	fn load(&self, query: Q, log: Logger) -> mpsc::Receiver<AudioResultData>;
}

/// Provides loading of audio pronunciation data.
///
/// This maintains a list of [AudioSource]s to use when loading a pronunciation
/// and manages caching and de-duping of requests.
pub struct AudioLoader<Q: AudioQuery> {
	sources: Arc<Vec<Box<dyn AudioSource<Q>>>>,
	jobs:    Arc<Mutex<HashMap<Q, AudioJob>>>,
	cache:   Arc<Mutex<AudioCache>>,
}

impl<Q: AudioQuery> AudioLoader<Q> {
	pub fn new(base_path: &Path) -> AudioLoader<Q> {
		let cache = AudioCache {
			cache:     util::Cache::new(),
			sources:   Vec::new(),
			base_path: base_path.to_owned(),
		};
		AudioLoader {
			sources: Default::default(),
			jobs:    Default::default(),
			cache:   Arc::new(Mutex::new(cache)),
		}
	}

	pub fn query(&self, log: &Logger, query: Q, force_reload: bool) -> AudioJob {
		let mut jobs = self.jobs.lock().unwrap();
		let job = if let Some(job) = jobs.get(&query) {
			Some(job.clone())
		} else {
			None
		};

		let job = if let Some(job) = job {
			// Since there is no guarantee when a completed job will be cleared
			// from the `jobs` map (if ever), if the job is already complete we
			// want to start a new one to prevent returning stale results.
			//
			// Audio loading is already cached in disk and loaded entries
			// already cached in memory with a TTL, so we don't need another
			// level of caching here.
			//
			// Note that we don't care about the race condition of the job
			// completing between now and the time we call start. In this case
			// the `start` will be a no-op and the results will be fresh enough.
			if job.is_complete() {
				jobs.remove(&query);
				None
			} else {
				Some(job)
			}
		} else {
			None
		};

		// Create a new job instance if necessary.
		let job = match job {
			Some(job) => job,
			_ => {
				let inner = AudioJobInner {
					query:   query.clone(),
					cache:   self.cache.clone(),
					sources: self.sources.clone(),
					log:     log.clone(),
					state:   Default::default(),
				};
				let job = AudioJob {
					inner: Arc::new(Mutex::new(Box::new(inner))),
				};
				jobs.insert(query, job.clone());
				job
			}
		};

		if force_reload {
			// Force the job to reload. Calling this on a completed job with
			// cached results will cause it to restart and reload everything.
			job.force_load();
		}

		// Start it! This is a no-op when not called for the first time.
		job.start();

		job
	}
}

/// Executes an [AudioQuery] and stores the completed results providing
/// synchronization and caching.
#[derive(Clone)]
pub struct AudioJob {
	/// This is the actual implementation for the Job.
	///
	/// There is at most one instance of any given job for a query `Q`, and this
	/// instance is shared between the respective [AudioJob] instances.
	///
	/// The [AudioJob] instance is just a container for this inner job. This
	/// allows us to decouple the type parameter `Q` from the actual job and
	/// to freely clone and pass instances of [AudioJob] around without
	/// worrying with lifetimes and such.
	inner: Arc<Mutex<Box<dyn AudioJobImpl>>>,
}

impl AudioJob {
	/// Blocks until the job is completed, returning all available results.
	pub fn wait(&self) -> Arc<AudioResult> {
		panic!()
	}

	/// Waits until there is some [AudioData] available in the result or the
	/// job completes.
	///
	/// Returns the [AudioData] as soon as it is available, or `None` if the
	/// request completed without loading any data.
	pub fn wait_any(&self) -> Option<AudioInfo> {
		panic!()
	}

	/// Is `true` if the job has already completed.
	///
	/// This is meant as an imprecise check to decide whether to start a new
	/// job or keep the currently running one. It is not meant to provide
	/// precise synchronization.
	fn is_complete(&self) -> bool {
		self.inner.lock().unwrap().is_complete()
	}

	/// Force this job to load from source even if a cached result is available.
	///
	/// If this is called after a job has already completed from cache, this
	/// will cause it to reload.
	fn force_load(&self) {
		self.inner.lock().unwrap().force_load();
	}

	/// This will start the job the first time it is called.
	///
	/// For a completed or running job this is a no-op. As such this method can
	/// be called at any point in the job lifetime.
	fn start(&self) {
		self.inner.lock().unwrap().start();
	}
}

trait AudioJobImpl: Send + Sync {
	fn is_complete(&self) -> bool;
	fn force_load(&mut self);
	fn start(&mut self);
}

/// Inner implementation for an [AudioJob].
///
/// NOTE: the access to this type is synchronized through a mutex.
struct AudioJobInner<Q: AudioQuery> {
	query:   Q,
	sources: Arc<Vec<Box<dyn AudioSource<Q>>>>,
	cache:   Arc<Mutex<AudioCache>>,
	log:     Logger,

	state: Arc<Mutex<AudioJobState>>,
}

struct AudioJobState {
	started:    bool,
	force_load: bool,
	completed:  bool,
	was_cached: bool,
}

impl Default for AudioJobState {
	fn default() -> AudioJobState {
		AudioJobState {
			started:    false,
			force_load: false,
			completed:  false,
			was_cached: false,
		}
	}
}

impl<Q: AudioQuery> AudioJobImpl for AudioJobInner<Q> {
	fn is_complete(&self) -> bool {
		self.state.lock().unwrap().completed
	}

	fn force_load(&mut self) {
		let mut state = self.state.lock().unwrap();
		let need_restart = state.completed && state.was_cached;
		state.force_load = true;
		if need_restart {
			state.started = false; // Force start to go again
		}
		drop(state);
		if need_restart {
			self.start();
		}
	}

	fn start(&mut self) {
		lazy_static! {
			static ref ENTRY_RE: Regex = Regex::new(r"^(?P<hash>[0-9a-f]{64})\.mp3$").unwrap();
		}

		let mut state = self.state.lock().unwrap();

		if !state.started {
			state.started = true;
			drop(state);

			// We don't want to keep any reference to self in the thread
			let query = self.query.clone();
			let sources = self.sources.clone();
			let cache = self.cache.clone();
			let log = self.log.clone();
			let state = self.state.clone();

			spawn(move || {
				let query_hash = query.query_hash();

				let log = log.new(o!("query" => query.query_info(), "hash" => query_hash.clone()));
				trace!(log, "starting query");

				let mut need_restart = true;
				while need_restart {
					// Load the cached entry. This will create a new empty entry
					let cached = cache.lock().unwrap().load(&log, &query_hash);

					let force_load = state.lock().unwrap().force_load;

					// Load any entry that is not available on the cached entry, or
					// if `force_load` is true then reload all sources.

					let mut new_entry = AudioCacheEntry::default(); // New cache entries
					let mut handles = Vec::new(); // Worker thread handles for joining
					let (tx_dat, rx_dat) = mpsc::channel(); // Receive the loaded audio entries
					let (tx_src, rx_src) = mpsc::channel(); // Receive the src metadata

					// Lock the cache entry
					let entry = cached.lock().unwrap();

					let mut loaded_from_cache = false;
					for src in sources.iter() {
						let src = src.copy();
						let id = src.id();

						new_entry.data_by_src.insert(id, Default::default());
						new_entry.info_by_src.insert(
							id,
							AudioSourceMetadata {
								source: id.to_string(),
								..Default::default()
							},
						);

						// Skip source if we don't need to load it
						if !force_load {
							if let Some(entry) = entry.as_ref() {
								if let Some(data) = entry.data_by_src.get(&id) {
									// If there is any data available at all, skip
									// loading.
									if data.len() > 0 {
										loaded_from_cache = true;
										continue;
									}

									// If there is no data available, but also no
									// errors, we can also skip (empty results).
									if let Some(meta) = entry.info_by_src.get(&id) {
										if meta.errors.len() == 0 {
											loaded_from_cache = true;
											continue;
										}
									}
								}
							}
						}

						let log = log.new(o!("worker" => id));
						let query = query.clone();
						let (tx_dat, tx_src) = (tx_dat.clone(), tx_src.clone());
						let handle = spawn(move || {
							time!(t_load);
							for it in src.load(query, log) {
								tx_dat.send((id, it)).unwrap();
							}
							tx_src.send((id, t_load.elapsed().as_secs_f64())).unwrap();
						});
						handles.push(handle);
					}

					drop(entry);
					drop(cached);
					drop(tx_dat);
					drop(tx_src);

					for (id, result) in rx_dat {
						match result {
							Ok(data) => {
								let entry = new_entry.data_by_src.get_mut(id).unwrap();
								entry.push(data);
							}
							Err(err) => {
								let entry = new_entry.info_by_src.get_mut(id).unwrap();
								entry.errors.push(format!("{}", err));
							}
						}
					}

					for (id, elapsed) in rx_src {
						let entry = new_entry.info_by_src.get_mut(id).unwrap();
						entry.elapsed = elapsed;
					}

					for it in handles {
						it.join().unwrap();
					}

					cache.lock().unwrap().merge(&log, query_hash.clone(), new_entry);

					let mut state = state.lock().unwrap();
					need_restart = state.force_load && loaded_from_cache;
					state.was_cached = loaded_from_cache;
					state.completed = true;
				}
			});
		}
	}
}

/// Provides in-memory caching for audio data.
///
/// This implementation makes a few assumptions:
/// - For any given `base_path`, there is a single instance of [AudioCache]
///   which has exclusive access to that directory.
/// - All access to this [AudioCache] is protected by a mutex.
struct AudioCache {
	sources:   Vec<(&'static str, &'static str)>,
	cache:     util::Cache<String, Mutex<Option<AudioCacheEntry>>>,
	base_path: PathBuf,
}

/// File name for the metadata file in a cache entry.
const CACHE_ENTRY_META_FILE: &'static str = "meta.json";

/// TTL for entries in the audio cache.
const AUDIO_CACHE_ENTRY_TTL: std::time::Duration = std::time::Duration::from_secs(15 * 60); // Entries are cached for 15 minutes

/// Cached entry in [AudioCache].
///
/// Entries are cached by [AudioQuery::query_hash] and store the whole result
/// data.
struct AudioCacheEntry {
	/// Audio data results indexed per [AudioSource].
	data_by_src: HashMap<&'static str, Vec<AudioData>>,
	/// Audio source metadata.
	info_by_src: HashMap<&'static str, AudioSourceMetadata>,
}

impl Default for AudioCacheEntry {
	fn default() -> AudioCacheEntry {
		AudioCacheEntry {
			data_by_src: Default::default(),
			info_by_src: Default::default(),
		}
	}
}

/// Metadata for the results from an [AudioSource].
#[derive(Serialize, Deserialize)]
struct AudioSourceMetadata {
	/// Source ID.
	pub source: String,
	/// List of errors when loading this result.
	pub errors: Vec<String>,
	/// Time elapsed loading all results from this source.
	pub elapsed: f64,
	/// Date and time for the results.
	pub timestamp: util::DateTime,
}

impl Default for AudioSourceMetadata {
	fn default() -> AudioSourceMetadata {
		AudioSourceMetadata {
			source:    Default::default(),
			errors:    Default::default(),
			elapsed:   Default::default(),
			timestamp: util::DateTime::now(),
		}
	}
}

impl AudioCache {
	/// Merge cache data with any currently available data and save it to disk.
	///
	/// Returns the actual cached entry, with the merged data.
	///
	/// If persisting the cache entry to disk fails, this will still cache the
	/// merged data in memory and log an error.
	pub fn merge(
		&self,
		log: &Logger,
		query_hash: String,
		to_merge: AudioCacheEntry,
	) -> Arc<Mutex<Option<AudioCacheEntry>>> {
		let log = log.new(o!("saving" => query_hash.clone()));
		let cache_entry = self.load(&log, &query_hash);
		let entry = cache_entry.clone();
		let mut entry = entry.lock().unwrap();

		let (_, full_cache_path) = self.cache_path(&query_hash);

		// Create the top-level cache entry directory, if it does not exist
		let mut failed_root = false;
		if entry.is_none() {
			*entry = Some(AudioCacheEntry::default());
			if let Err(err) = fs::create_dir_all(&full_cache_path) {
				error!(log, "creating entry directory: {}", err);
				failed_root = true;
			}
		}

		fn create_src_dir(log: &Logger, src: &'static str, base_path: &Path) -> (bool, PathBuf) {
			let src_path = base_path.join(src);
			if let Err(err) = fs::create_dir_all(&src_path) {
				error!(log, "creating `{}` directory: {}", src, err);
				(false, src_path)
			} else {
				(true, src_path)
			}
		}

		let entry = entry.as_mut().unwrap();
		let mut src_dir_status = HashMap::new();

		for (src, mut val) in to_merge.info_by_src {
			let val_ref = &mut val;
			let meta = entry
				.info_by_src
				.entry(src)
				.and_modify(|e| {
					e.elapsed = val_ref.elapsed;
					e.timestamp = val_ref.timestamp.clone();
					e.errors.append(&mut val_ref.errors);
				})
				.or_insert(val);

			// We only write to the disk if the root directory has been created
			if !failed_root {
				// Create the top level directory for this source
				let (ok, src_path) = create_src_dir(&log, src, &full_cache_path);
				if ok {
					// Write the metadata
					let meta_path = src_path.join(CACHE_ENTRY_META_FILE);
					if let Err(err) = util::write_json(meta_path, meta) {
						error!(log, "writing `{}` metadata: {}", src, err);
					}
				}
				// Record the source directory creation status for the entries
				// loop below.
				src_dir_status.insert(src, (ok, src_path));
			}
		}

		for (src, new_entries) in to_merge.data_by_src {
			// Update the entry information in the cached entry
			let (can_write, src_path) = match src_dir_status.get(src) {
				Some(pair) => (pair.0, pair.1.clone()),
				None => {
					// The source was never created because `new_entry` did not
					// specify a metadata for it
					create_src_dir(&log, src, &full_cache_path)
				}
			};

			let entries = entry.data_by_src.entry(src).or_insert(Vec::new());
			for AudioData(new_data, new_hash) in new_entries {
				// Write or re-write the cached entry
				if can_write {
					let entry_path = src_path.join(format!("{}.mp3", new_hash));
					if let Err(err) = fs::write(&entry_path, &new_data) {
						error!(log, "writing `{}` entry: {}", new_hash, err);
					}
				}

				// Add a new AudioData to the cache entries
				if !entries.iter().any(|AudioData(_data, hash)| hash == &new_hash) {
					entries.push(AudioData(new_data, new_hash));
				}
			}
		}

		cache_entry
	}

	/// Loads the cached entry for a given query hash.
	///
	/// If the entry is not available in the cache, it will be loaded from disk
	/// and cached in memory.
	pub fn load(&self, log: &Logger, query_hash: &String) -> Arc<Mutex<Option<AudioCacheEntry>>> {
		lazy_static! {
			static ref ENTRY_RE: Regex = Regex::new(r"^(?P<hash>[0-9a-f]{64})\.mp3$").unwrap();
		}

		if let Some(res) = self.cache.get_and_renew(&query_hash, AUDIO_CACHE_ENTRY_TTL) {
			trace!(log, "loaded {} from memory cache", query_hash);
			res.clone()
		} else {
			// Try to load an entry from the file system:
			time!(t_cache);

			let (cache_path, full_cache_path) = self.cache_path(&query_hash);

			let entry = if !full_cache_path.exists() {
				None
			} else {
				let mut entry = AudioCacheEntry::default();

				// Load available entries
				for (id, _name) in self.sources.iter() {
					// Directory for this source in `full_cache_path`
					let base_path = full_cache_path.join(id);

					// Try to load the source metadata
					let meta_path = base_path.join(CACHE_ENTRY_META_FILE);
					let mut meta: AudioSourceMetadata = match util::read_json(&meta_path) {
						Ok(Some(data)) => data,
						Ok(None) => {
							// We don't have an audio metadata, but still want
							// to load any audio entries
							AudioSourceMetadata {
								source: id.to_string(),
								..Default::default()
							}
						}
						Err(err) => AudioSourceMetadata {
							source:    id.to_string(),
							errors:    vec![format!("{}", err)],
							elapsed:   0.0,
							timestamp: util::DateTime::now(),
						},
					};

					// Read all entries in the source's cache directory
					let mut entries = Vec::new();
					match fs::read_dir(&base_path) {
						Ok(dir) => {
							for entry in dir {
								match entry {
									Ok(entry) => {
										let mut valid = false;
										if let Some(name) = entry.file_name().to_str() {
											if name == CACHE_ENTRY_META_FILE {
												continue;
											}

											if let Some(caps) = ENTRY_RE.captures(name) {
												valid = true;

												let hash = caps.name("hash").unwrap().as_str();
												let data = match std::fs::read(entry.path()) {
													Ok(data) => AudioData(data, hash.to_string()),
													Err(err) => {
														meta.errors.push(format!("{}", err));
														continue;
													}
												};
												entries.push(data);
											}
										}
										if !valid {
											warn!(
												log,
												"invalid cache entry: {}/{}/{}",
												cache_path,
												id,
												entry.file_name().to_string_lossy()
											);
										}
									}
									Err(err) => meta.errors.push(format!("{}", err)),
								}
							}
						}
						Err(err) => meta.errors.push(format!("{}", err)),
					}

					// Save source data to cache entry
					entry.info_by_src.insert(id, meta);
					entry.data_by_src.insert(id, entries);
				}

				trace!(
					log,
					"loaded cached entries for {}",
					entry.info_by_src.keys().join("/");
					t_cache
				);

				Some(entry)
			};

			// Store the entry in the cache and return it
			self.cache
				.save(query_hash.clone(), Mutex::new(entry), AUDIO_CACHE_ENTRY_TTL)
		}
	}

	/// Returns the relative and full path to the cache entry directory for
	/// the given query hash.
	fn cache_path(&self, query_hash: &str) -> (String, PathBuf) {
		let cache_path = format!("{}/{}", &query_hash[0..2], &query_hash);
		let mut full_path = self.base_path.to_owned();
		for p in cache_path.split('/') {
			full_path.push(p);
		}
		(cache_path, full_path)
	}
}

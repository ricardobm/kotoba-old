//! Support for audio pronunciation loading.

use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::time::{Duration, Instant};

use slog::Logger;

use itertools::*;
use regex::Regex;

use util;

/// Completed result for an [AudioQuery].
pub struct AudioResult {
	pub items:   HashMap<&'static str, Vec<AudioInfo>>,
	pub sources: Vec<AudioSourceMetadata>,
}

impl AudioResult {
	pub fn has_items(&self) -> bool {
		self.items.values().any(|x| x.len() > 0)
	}

	fn from_cache_entry(base_path: &str, entry: &AudioCacheEntry) -> AudioResult {
		let mut result = AudioResult {
			items:   Default::default(),
			sources: Default::default(),
		};

		for (src, val) in entry.data_by_src.iter() {
			let mut entries = vec![];
			for it in val.iter() {
				entries.push(AudioInfo {
					source: src,
					hash:   it.hash.clone(),
					file:   format!("{}/{}/{}.mp3", base_path, src, it.hash),
					cached: it.cached,
				});
			}
			result.items.insert(src, entries);
		}

		for meta in entry.info_by_src.values() {
			result.sources.push(meta.clone())
		}
		result.sources.sort_by_key(|x| x.index);

		result
	}
}

/// Single item returned by an [AudioSource]
#[allow(dead_code)]
pub type AudioDataResult = util::Result<AudioData>;

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

/// Audio data.
pub struct AudioData {
	pub data:   Vec<u8>,
	pub hash:   String,
	pub cached: bool,
}

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
	fn load(&self, query: Q, log: Logger, id: SourceId, sink: mpsc::Sender<(SourceId, AudioDataResult)>);
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
	pub fn new(base_path: &Path, sources: Vec<Box<dyn AudioSource<Q>>>) -> AudioLoader<Q> {
		let mut cache_sources = Vec::new();
		for it in sources.iter() {
			cache_sources.push((it.id(), it.name()));
		}
		let cache = AudioCache {
			cache:     util::Cache::new(),
			sources:   cache_sources,
			base_path: base_path.to_owned(),
		};
		AudioLoader {
			sources: Arc::new(sources),
			jobs:    Default::default(),
			cache:   Arc::new(Mutex::new(cache)),
		}
	}

	pub fn from_cache(&self, log: &Logger, query_hash: &str, src: &str, hash: &str) -> Option<Vec<u8>> {
		let entry = self.cache.lock().unwrap().load_with_optional_save(log, query_hash, false);
		let entry = entry.lock().unwrap();
		if let Some(entries) = entry.data_by_src.get(src) {
			entries.iter().filter(|x| &x.hash == hash).map(|x| x.data.clone()).next()
		} else {
			None
		}
	}

	pub fn query(&self, log: &Logger, query: Q, force_reload: bool) -> AudioJob {
		let mut jobs = self.jobs.lock().unwrap();

		// Retrieve an existing job for this query.
		let job = if let Some(job) = jobs.get(&query) {
			Some(job.clone())
		} else {
			None
		};

		let job = if let Some(job) = job {
			// Check if the job is stale or completed. If it is completed we
			// want to start a new job (since this is a new request) and since
			// we don't purge completed jobs from the map (for simplicity sake).
			//
			// Audio loading is already cached in disk and loaded entries
			// already cached in memory with a TTL, so we don't need another
			// level of caching here.
			//
			// We also check the timeout on the job to prevent a stale timeout
			// job from blocking requests forever. Having multiple jobs active
			// for the same request is not good, but it is better than never
			// finishing a particular query.
			//
			// Having multiple jobs is safe, since the final cache write is
			// protected by a global lock and results end-up merged.
			//
			// Note that we don't care about the race condition of the job
			// completing between now and the time we call start. In this case
			// the `start` will be a no-op and the results will be fresh enough.
			let (started_at, completed) = job.get_status();
			if completed || (Instant::now() - started_at) > AUDIO_JOB_TIMEOUT {
				// Clear the stale job.
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
					inner: Arc::new(Box::new(inner)),
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
	/// This contains the actual implementation for the Job.
	///
	/// There is usually one instance of any given job for a query `Q`, and this
	/// instance is shared between the respective [AudioJob] instances. This
	/// managing is done by the [AudioLoader<Q>].
	///
	/// The outer [AudioJob] just provides a query-independent wrapper for the
	/// inner job.
	inner: Arc<Box<dyn AudioJobImpl>>,
}

/// Global timeout on any [AudioJob].
///
/// A new job will be started even if there is a pending one that has been
/// started for longer than this.
const AUDIO_JOB_TIMEOUT: Duration = Duration::from_secs(15);

impl AudioJob {
	/// Blocks until the job is completed, returning all available results.
	pub fn wait(&self) -> AudioResult {
		self.inner.wait()
	}

	/// Waits until there is some [AudioData] available in the result or the
	/// job completes.
	///
	/// Returns the available results, if any.
	pub fn wait_any(&self) -> AudioResult {
		self.inner.wait_any()
	}

	/// Returns the job created time and its completion flag.
	///
	/// This is provided as a way to check the status of a job and decide if
	/// a new one can be spawned. It is not meant for precise synchronization
	/// since there is no lock on status changes after this call returns.
	fn get_status(&self) -> (Instant, bool) {
		self.inner.get_status()
	}

	/// Force this job to load from source even if a cached result is available.
	///
	/// If this is called after a job has already completed from cache, this
	/// will cause it to reload.
	fn force_load(&self) {
		self.inner.force_load();
	}

	/// This will start the job the first time it is called.
	///
	/// For a completed or running job this is a no-op. As such this method can
	/// be called at any point in the job lifetime.
	fn start(&self) {
		self.inner.start();
	}
}

trait AudioJobImpl: Send + Sync {
	fn wait(&self) -> AudioResult;
	fn wait_any(&self) -> AudioResult;
	fn get_status(&self) -> (Instant, bool);
	fn force_load(&self);
	fn start(&self);
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
	started:     bool,
	created_at:  Instant,
	force_load:  bool,
	completed:   bool,
	was_cached:  bool,
	has_results: bool,
	loaded_one:  util::Condition,
	loaded_all:  util::Condition,
}

impl Default for AudioJobState {
	fn default() -> AudioJobState {
		AudioJobState {
			started:     false,
			created_at:  Instant::now(),
			force_load:  false,
			completed:   false,
			was_cached:  false,
			has_results: false,
			loaded_one:  Default::default(),
			loaded_all:  Default::default(),
		}
	}
}

impl<Q: AudioQuery> AudioJobImpl for AudioJobInner<Q> {
	fn wait(&self) -> AudioResult {
		let loaded_all = {
			let state = self.state.lock().unwrap();
			state.loaded_all.clone()
		};
		loaded_all.wait();

		let query_hash = self.query.query_hash();
		let entry = self.cache.lock().unwrap().load(&self.log, &query_hash);
		let entry = entry.lock().unwrap();
		AudioResult::from_cache_entry(&entry.base_path, &entry)
	}

	fn wait_any(&self) -> AudioResult {
		let loaded_one = {
			let state = self.state.lock().unwrap();
			state.loaded_one.clone()
		};
		loaded_one.wait();

		let query_hash = self.query.query_hash();
		let entry = self.cache.lock().unwrap().load(&self.log, &query_hash);
		let entry = entry.lock().unwrap();
		AudioResult::from_cache_entry(&entry.base_path, &entry)
	}

	fn get_status(&self) -> (Instant, bool) {
		let state = self.state.lock().unwrap();
		(state.created_at, state.completed)
	}

	fn force_load(&self) {
		let mut state = self.state.lock().unwrap();

		// Force loading from sources. There are three scenarios for us setting
		// this:
		//
		// - Before [start] checks the `force_load` flag:
		//   - [start] will pick up on the value and force loading.
		// - After [start] checked the `force_load` flag, before it completes:
		//   - [start] has a check at the end that will cause it to restart.
		// - After [start] has already completed:
		//   - we need to restart manually.
		state.force_load = true;

		// If job was completed with cached entries we need to restart it.
		let need_restart = state.completed && state.was_cached;

		// If we need to restart...
		if need_restart {
			// force a call to [start] to spawn a new thread
			state.started = false;

			// reset the "loaded" conditions
			state.loaded_all.reset();
			if !state.has_results {
				// give the "loaded one" condition another shot since we are
				// reloading
				state.loaded_one.reset();
			}

			// Release the state lock on the state and call start again
			drop(state);
			self.start();
		}
	}

	fn start(&self) {
		lazy_static! {
			static ref ENTRY_RE: Regex = Regex::new(r"^(?P<hash>[0-9a-f]{64})\.mp3$").unwrap();
		}

		let mut state = self.state.lock().unwrap();

		if !state.started {
			state.started = true;
			drop(state); // We don't want to keep the state locked for long

			// We don't want to keep any reference to self in the thread
			let query = self.query.clone();
			let sources = self.sources.clone();
			let cache = self.cache.clone();
			let log = self.log.clone();
			let state = self.state.clone();

			// Spawn a new thread for loading.
			spawn(move || {
				let query_hash = query.query_hash();

				let log = log.new(o!("query" => query.query_info(), "hash" => query_hash.clone()));
				trace!(log, "starting query");

				let mut need_restart = true;
				while need_restart {
					// Load the cached entry. This will create a new empty entry
					// if it does not exist.
					let cached_entry = cache.lock().unwrap().load(&log, &query_hash);

					// Check if we are forced to load from source.
					let force_load = state.lock().unwrap().force_load;

					// Determine what sources do we need to load from and spawn
					// workers to load them:

					let mut has_cached_entries = false;
					let cached_entry = cached_entry.lock().unwrap();
					let mut has_results = cached_entry.has_results();

					let mut handles = Vec::new(); // Worker thread handles for joining
					let (tx_dat, rx_dat) = mpsc::channel(); // Receive the loaded audio entries
					let (tx_src, rx_src) = mpsc::channel(); // Receive the src metadata

					let mut src_metadata = HashMap::new();

					for (index, src) in sources.iter().enumerate() {
						let audio_src = src.copy();
						let src = SourceId {
							id:    src.id(),
							name:  src.name(),
							index: index,
						};

						// Check if we can skip the source
						if !force_load {
							if let Some(data) = cached_entry.data_by_src.get(&src.id) {
								// If there is any data available at all,
								// skip loading.
								if data.len() > 0 {
									has_cached_entries = true;
									continue;
								}

								// If there is no data available, but also no
								// errors, we can also skip (empty results).
								if let Some(meta) = cached_entry.info_by_src.get(&src.id) {
									if meta.errors.len() == 0 {
										has_cached_entries = true;
										continue;
									}
								}
							}
						}

						// Setup the metadata entry for this source
						src_metadata.insert(src.id, AudioSourceMetadata::new(&src));

						// Spawn a worker thread to load from the source
						let log = log.new(o!("worker" => src.id));
						let query = query.clone();
						let (tx_dat, tx_src) = (tx_dat.clone(), tx_src.clone());
						let handle = spawn(move || {
							time!(t_load);

							audio_src.load(query, log, src.clone(), tx_dat);

							// Send the metadata (for now just the elapsed time)
							tx_src.send((src, t_load.elapsed().as_secs_f64())).unwrap();
						});
						handles.push(handle);
					}

					// don't keep the cache entry lock while we wait for results
					drop(cached_entry);

					// drop those here otherwise the receivers will not close
					drop(tx_dat);
					drop(tx_src);

					// Consume the results from all workers and add cache it
					for (src, result) in rx_dat {
						let is_ok = result.is_ok();
						cache.lock().unwrap().merge_result(&log, &src, &query_hash, result);
						if is_ok {
							has_results = true;
							state.lock().unwrap().loaded_one.trigger();
						}
					}

					// Consume the result metadata and add cache it
					for (src, elapsed) in rx_src {
						let mut metadata = src_metadata.remove(src.id).unwrap();
						metadata.elapsed = elapsed;
						cache.lock().unwrap().merge_metadata(&log, &src, &query_hash, metadata);
					}

					// Join all threads
					for it in handles {
						it.join().unwrap();
					}

					// If could happen that the `force_load` flag was updated
					// after we checked it.
					//
					// In that case, the complete flag was not set, so the job
					// would not attempt to restart the load and we need to do
					// it ourselves.
					let mut state = state.lock().unwrap();
					need_restart = state.force_load && has_cached_entries;
					if !need_restart {
						state.was_cached = has_cached_entries;
						state.has_results = has_results;
						state.completed = true;
						state.loaded_one.trigger();
						state.loaded_all.trigger();
					}
				}
			});
		}
	}
}

/// Provides in-memory caching for audio data and manages the disk cache.
///
/// This implementation makes a few assumptions:
/// - For any given `base_path`, there is a single instance of [AudioCache]
///   which has exclusive access to that directory.
/// - All access to this [AudioCache] is protected by a mutex.
struct AudioCache {
	sources:   Vec<(&'static str, &'static str)>,
	cache:     util::Cache<String, Mutex<AudioCacheEntry>>,
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
	/// Base path for this entry, relative to the cache base path.
	base_path: String,
	/// Audio data results indexed per [AudioSource].
	data_by_src: HashMap<&'static str, Vec<AudioData>>,
	/// Audio source metadata.
	info_by_src: HashMap<&'static str, AudioSourceMetadata>,
}

impl AudioCacheEntry {
	fn has_results(&self) -> bool {
		self.data_by_src.values().any(|x| x.len() > 0)
	}
}

impl AudioCacheEntry {
	fn new(base_path: String) -> AudioCacheEntry {
		AudioCacheEntry {
			base_path:   base_path,
			data_by_src: Default::default(),
			info_by_src: Default::default(),
		}
	}
}

/// Metadata for the results from an [AudioSource].
#[derive(Clone, Serialize, Deserialize)]
pub struct AudioSourceMetadata {
	/// Source ID.
	pub source: String,
	/// Source name.
	pub name: String,
	/// Index of this source in the priority list.
	pub index: usize,
	/// List of errors when loading this result.
	pub errors: Vec<String>,
	/// Time elapsed loading all results from this source.
	pub elapsed: f64,
	/// Date and time for the results.
	pub timestamp: util::DateTime,
}

#[derive(Clone)]
pub struct SourceId {
	id:    &'static str,
	name:  &'static str,
	index: usize,
}

impl AudioSourceMetadata {
	fn new(source: &SourceId) -> AudioSourceMetadata {
		AudioSourceMetadata {
			source:    source.id.to_string(),
			name:      source.name.to_string(),
			index:     source.index,
			errors:    Default::default(),
			elapsed:   Default::default(),
			timestamp: util::DateTime::now(),
		}
	}

	fn new_with_error(source: &SourceId, err: String) -> AudioSourceMetadata {
		let mut out = Self::new(source);
		out.errors.push(err);
		out
	}
}

impl AudioCache {
	/// Merge a single audio result into the cache.
	///
	/// NOTE: this does not persist metadata information.
	pub fn merge_result(&self, log: &Logger, source: &SourceId, query_hash: &String, data: AudioDataResult) {
		let log = log.new(o!("saving" => format!("{}/{}", query_hash, source.id)));
		let entry = self.load(&log, query_hash);
		let mut entry = entry.lock().unwrap();

		match data {
			Ok(audio) => {
				// Save the audio data to the disk
				if let Some(cache_path) = self.get_entry_dir(&log, source.id, query_hash) {
					let entry_path = cache_path.join(format!("{}.mp3", audio.hash));
					if let Err(err) = fs::write(&entry_path, &audio.data) {
						error!(log, "writing `{}` entry: {}", audio.hash, err);
					}
					trace!(log, "saved {} successfully", audio.hash);
				}

				// Insert the data in the cache entry
				let entries = entry.data_by_src.entry(source.id).or_insert(vec![]);
				if !entries.iter().any(|cur| cur.hash == audio.hash) {
					entries.push(audio);
				}

				// Make sure we have a metadata entry in the cache
				entry
					.info_by_src
					.entry(source.id)
					.or_insert_with(|| AudioSourceMetadata::new(source));
			}
			Err(err) => {
				// Append the error to the source metadata entry
				let metadata = entry
					.info_by_src
					.entry(source.id)
					.or_insert_with(|| AudioSourceMetadata::new(source));
				metadata.errors.push(format!("{}", err));
			}
		}
	}

	/// Merge metadata information into the cache and persists the metadata file.
	pub fn merge_metadata(&self, log: &Logger, source: &SourceId, query_hash: &String, mut meta: AudioSourceMetadata) {
		let log = log.new(o!("saving" => format!("{}/{}/{}", query_hash, source.id, CACHE_ENTRY_META_FILE)));
		let entry = self.load(&log, &query_hash);
		let mut entry = entry.lock().unwrap();
		let meta_ref = &mut meta;

		// Make sure we have a data entry for this source (in case it returns empty)
		entry.data_by_src.entry(source.id).or_insert(Default::default());

		// Insert the metadata
		let metadata = entry
			.info_by_src
			.entry(source.id)
			.and_modify(|e| {
				e.errors.append(&mut meta_ref.errors);
				e.elapsed = meta_ref.elapsed;
				e.timestamp = meta_ref.timestamp.clone();
			})
			.or_insert(meta);

		if let Some(cache_path) = self.get_entry_dir(&log, source.id, query_hash) {
			let meta_path = cache_path.join(CACHE_ENTRY_META_FILE);
			if let Err(err) = util::write_json(meta_path, metadata) {
				error!(log, "writing metadata: {}", err);
			}
		}
	}

	fn get_entry_dir(&self, log: &Logger, src: &'static str, query_hash: &str) -> Option<PathBuf> {
		let (_, cache_path) = self.cache_path(query_hash);
		let cache_path = cache_path.join(src);
		match fs::create_dir_all(&cache_path) {
			Ok(_) => Some(cache_path),
			Err(err) => {
				error!(log, "creating entry directory: {}", err);
				None
			}
		}
	}

	/// Loads the cached entry for a given query hash.
	///
	/// If the entry is not available in the cache, it will be loaded from disk
	/// and cached in memory.
	pub fn load(&self, log: &Logger, query_hash: &str) -> Arc<Mutex<AudioCacheEntry>> {
		self.load_with_optional_save(log, query_hash, true)
	}

	pub fn load_with_optional_save(&self, log: &Logger, query_hash: &str, save: bool) -> Arc<Mutex<AudioCacheEntry>> {
		lazy_static! {
			static ref ENTRY_RE: Regex = Regex::new(r"^(?P<hash>[0-9a-f]{64})\.mp3$").unwrap();
		}

		// TODO: remove this weirdness
		let query_hash = query_hash.to_string();
		let query_hash = &query_hash;

		if let Some(res) = self.cache.get_and_renew(query_hash, AUDIO_CACHE_ENTRY_TTL) {
			trace!(log, "loaded {} from memory cache", query_hash);
			res.clone()
		} else {
			// Try to load an entry from the file system:
			time!(t_cache);

			let (cache_path, full_cache_path) = self.cache_path(&query_hash);

			let entry = if !full_cache_path.exists() {
				AudioCacheEntry::new(cache_path)
			} else {
				let mut entry = AudioCacheEntry::new(cache_path);

				// Load available entries
				for (index, (id, name)) in self.sources.iter().enumerate() {
					let src = SourceId { id, name, index };

					// Directory for this source in `full_cache_path`
					let base_path = full_cache_path.join(id);

					// Try to load the source metadata
					let meta_path = base_path.join(CACHE_ENTRY_META_FILE);
					let mut meta: AudioSourceMetadata = match util::read_json(&meta_path) {
						Ok(Some(data)) => data,
						Ok(None) => {
							// We don't have an audio metadata, but still want
							// to load any audio entries
							AudioSourceMetadata::new(&src)
						}
						Err(err) => AudioSourceMetadata::new_with_error(&src, format!("{}", err)),
					};

					// Read all entries in the source's cache directory
					let mut entries = Vec::new();
					match fs::read_dir(&base_path) {
						Ok(dir) => {
							for dir_entry in dir {
								match dir_entry {
									Ok(dir_entry) => {
										let mut valid = false;
										if let Some(name) = dir_entry.file_name().to_str() {
											if name == CACHE_ENTRY_META_FILE {
												continue;
											}

											if let Some(caps) = ENTRY_RE.captures(name) {
												valid = true;

												let hash = caps.name("hash").unwrap().as_str();
												let data = match std::fs::read(dir_entry.path()) {
													Ok(data) => AudioData {
														data:   data,
														hash:   hash.to_string(),
														cached: true,
													},
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
												entry.base_path,
												id,
												dir_entry.file_name().to_string_lossy()
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

				entry
			};

			// Store the entry in the cache and return it
			if save {
				self.cache
					.save(query_hash.clone(), Mutex::new(entry), AUDIO_CACHE_ENTRY_TTL)
			} else {
				Arc::new(Mutex::new(entry))
			}
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

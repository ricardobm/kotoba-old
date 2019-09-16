//! Support for audio pronunciation loading.

use std::hash::Hash;
use std::path::Path;
use std::sync::Arc;

use slog::Logger;

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
pub trait AudioQuery: Clone + Hash + Eq {
	/// SHA-256 hash for this query parameters.
	///
	/// This is used for determining the audio cached locations and to
	/// de-duplicate the queries.
	fn query_hash(&self) -> String;
}

/// Trait for a source that can be used to load audio data.
pub trait AudioSource<Q: AudioQuery>: Send + Sync {
	/// Short identifier for the source. Must be a valid path component.
	fn id(&self) -> &'static str;

	/// Human readable name for the source.
	fn name(&self) -> &'static str;

	/// Loads a query from this source.
	fn load(&self, query: Q, log: Logger) -> Box<dyn Iterator<Item = AudioResultData>>;
}

/// Provides loading of audio pronunciation data.
///
/// This maintains a list of [AudioSource]s to use when loading a pronunciation
/// and manages caching and de-duping of requests.
pub struct AudioLoader<Q: AudioQuery> {
	_sources: Vec<Box<dyn AudioSource<Q>>>,
}

impl<Q: AudioQuery> AudioLoader<Q> {
	pub fn new() -> AudioLoader<Q> {
		AudioLoader { _sources: Vec::new() }
	}

	pub fn query(&self, _query: Q, _force_reload: bool) -> AudioJob {
		panic!()
	}

	pub fn _base_path(&self) -> &Path {
		panic!()
	}
}

/// Stores the completed or intermediate results of an [AudioQuery] providing
/// synchronization to the complete result or the first audio.
#[derive(Clone)]
pub struct AudioJob {}

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
		None
	}
}

//! Audio API

/// Pronunciation API request arguments.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
	/// Optional identifier to include in the response.
	#[serde(default)]
	pub id: u64,

	/// The main expression to search.
	pub expression: String,

	/// Optional reading to lookup. Note that some sources won't return without
	/// a valid reading.
	///
	/// This will be converted to hiragana.
	#[serde(default)]
	pub reading: String, // TODO: query this from the dictionary when not available

	/// If `true` will force loading from the source even if there is a cached
	/// result available.
	///
	/// Even if this is `true`, cached entries will still be returned, alongside
	/// any entries loaded from the source.
	#[serde(default)]
	pub reload_sources: bool,

	/// If `true`, try to return a single result as fast as possible.
	///
	/// This will still trigger a full load in the background, even
	/// respecting the [reload_sources] flag, but the request itself will
	/// return as soon as a single source is available.
	///
	/// For cached requests this is no different than a normal load, since all
	/// cached entries will be loaded before returning.
	#[serde(default)]
	pub quick_load: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
	/// Identifier provided in the request.
	pub id: u64,

	/// The provided request expression, normalized.
	pub expression: String,

	/// The provided request reading, normalized and converted to hiragana.
	pub reading: String,

	/// Hash for the main request in the cache.
	pub cache_key: String,

	/// List of results. This is sorted from most relevant to least relevant.
	pub results: Vec<Item>,

	/// Metadata information about sources and the lookup loading.
	pub sources: Vec<Source>,

	/// Is `true` if the request has either of top level errors or source
	/// specific errors.
	pub has_errors: bool,

	/// List of top-level errors occurred during the request. Having errors
	/// does not necessarily prevent the request from having results.
	///
	/// Note that source specific errors are returned in their respective
	/// sources.
	pub errors: Vec<String>,

	/// Number of seconds the request took to process server-side.
	pub elapsed: f64,

	/// Log information for this request.
	pub log: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
	/// Source ID for this request.
	pub source: String,

	/// File name for this pronunciation file.
	pub name: String,

	/// Sound SHA-256 hash.
	pub hash: String,

	/// Relative URL to request for this pronunciation file.
	pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
	/// Source ID.
	pub source: String,

	/// Source human readable name,
	pub name: String,

	/// This is `true` if this source was loaded from cache.
	pub cached: bool,

	/// Total results from this source in the response.
	pub total_results: usize,

	/// Source specific errors, if any.
	///
	/// If the request was cached, these are the original errors. Note that
	/// error-ed requests are only cached if they provide at least one result.
	///
	/// Having errors does not necessarily prevent the request from having
	/// results.
	pub errors: Vec<String>,

	/// Number of seconds elapsed loading this source.
	pub elapsed: f64,

	/// Log information for this source.
	pub log: Vec<String>,
}

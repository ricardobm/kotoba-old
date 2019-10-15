//! Audio API

use regex::Regex;
use rocket::State;
use rocket_contrib::json::Json;

use app::App;
use audio;
use logging::RequestLog;

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
	pub reading: String,

	/// If `true` will force loading from the source even if there is a cached
	/// result available.
	///
	/// Even if this is `true`, cached entries will still be returned, alongside
	/// any entries loaded from the source.
	#[serde(default)]
	pub force_reload: bool,

	/// If `true`, try to return a single result as fast as possible.
	///
	/// This will still trigger a full load in the background, even
	/// respecting the [force_reload] flag, but the request itself will
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

	/// Is `true` if any of the sources has errors.
	pub has_errors: bool,
}

impl Response {
	pub fn append_result(&mut self, mut result: audio::AudioResult) {
		for it in result.sources {
			let total = match result.items.get(it.source.as_str()) {
				Some(items) => items.len(),
				None => 0,
			};
			if it.errors.len() > 0 {
				self.has_errors = true;
			}
			self.sources.push(Source {
				source:        it.source,
				name:          it.name,
				total_results: total,
				errors:        it.errors,
				elapsed:       it.elapsed,
			});
		}

		for it in self.sources.iter() {
			if let Some(entries) = result.items.remove(it.source.as_str()) {
				for it in entries {
					self.results.push(Item::from_audio_info(it));
				}
			}
		}

		// if for some reason the result contains non-indexes sources, add those.
		for (_, entries) in result.items {
			for it in entries {
				self.results.push(Item::from_audio_info(it));
			}
		}
	}
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

	/// This is `true` if this item was loaded from cache.
	pub cached: bool,
}

impl Item {
	fn from_audio_info(info: audio::AudioInfo) -> Item {
		// Skip the first directory in the file path (e.g. `05/05f3d31...`)
		use itertools::*;
		let file_name = info.file.splitn(2, '/').skip(1).join("/");
		Item {
			source: info.source.to_string(),
			name:   info.file.split('/').rev().next().unwrap().to_string(),
			hash:   info.hash,
			url:    format!("{}/{}", AUDIO_LOAD_BASE_PATH, file_name),
			cached: info.cached,
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
	/// Source ID.
	pub source: String,

	/// Source human readable name,
	pub name: String,

	/// Total results from this source in the response.
	pub total_results: usize,

	/// Total number of errors from this source in the response.
	pub errors: Vec<String>,

	/// Number of seconds elapsed loading this source.
	pub elapsed: f64,
}

const AUDIO_LOAD_BASE_PATH: &'static str = "/api/audio/get";

#[get("/audio/get/<query>/<src>/<file>")]
pub fn get_audio_file(
	log: RequestLog,
	app: State<&App>,
	query: String,
	src: String,
	file: String,
) -> AudioFileResponse {
	lazy_static! {
		static ref RE_HASH: Regex = Regex::new(r"^[a-z0-9]+$").unwrap();
		static ref RE_NAME: Regex = Regex::new(r"^[a-z]+$").unwrap();
		static ref RE_FILE: Regex = Regex::new(r"^[a-z0-9]+\.mp3$").unwrap();
	}

	let log = log.new(o!("audio" => format!("{}/{}/{}", query, src, file)));

	if !RE_HASH.is_match(&query) {
		AudioFileResponse::NotFound
	} else if !RE_NAME.is_match(&src) {
		AudioFileResponse::NotFound
	} else if !RE_FILE.is_match(&file) {
		AudioFileResponse::NotFound
	} else {
		let loader = app.japanese_audio();
		let hash = file.trim_end_matches(".mp3");
		match loader.from_cache(&log, &query, &src, &hash) {
			Some(data) => AudioFileResponse::File(file, data),
			None => AudioFileResponse::NotFound,
		}
	}
}

#[post("/audio/query", data = "<input>")]
pub fn query_audio(log: RequestLog, input: Json<Request>, app: State<&App>) -> Json<Response> {
	use audio::AudioQuery;
	use japanese::{Search, SearchMode, SearchOptions};
	use japanese::JapaneseAudioQuery;
	use kana::normalize_search_string;

	let expression = normalize_search_string(input.expression.trim(), false);
	let reading = input.reading.trim();
	let reading = if reading.len() == 0 {
		let db = app.db();
		let options = SearchOptions {
			mode: SearchMode::Is,
			..Default::default()
		};
		let (terms, _) = db.search_terms(&log, &expression, &options);
		let mut found_reading = None;
		for (_, it) in terms {
			if let Some(reading) = it.reading_for(&expression) {
				found_reading = Some(reading.clone());
			}
		}
		if let Some(reading) = found_reading {
			reading
		} else {
			String::new()
		}
	} else {
		normalize_search_string(reading, true)
	};

	let query = JapaneseAudioQuery { expression, reading };

	let mut response = Response {
		id:         input.id,
		expression: query.expression.clone(),
		reading:    query.reading.clone(),
		cache_key:  query.query_hash(),

		results:    Vec::new(),
		sources:    Vec::new(),
		has_errors: false,
	};

	let loader = app.japanese_audio();
	let worker = loader.query(&log, query, input.force_reload);

	let mut loaded = false;
	if input.quick_load {
		let result = worker.wait_any();
		if result.has_items() {
			response.append_result(result);
			loaded = true;
		}
	}

	if !loaded {
		response.append_result(worker.wait());
	}

	Json(response)
}

pub enum AudioFileResponse {
	File(String, Vec<u8>),
	NotFound,
}

use rocket::http::hyper::header::{Charset, ContentDisposition, DispositionParam, DispositionType};

impl<'r> rocket::response::Responder<'r> for AudioFileResponse {
	fn respond_to(self, _request: &rocket::Request) -> rocket::response::Result<'r> {
		use rocket::http::ContentType;
		use rocket::Response;

		let response = match self {
			AudioFileResponse::File(name, data) => Response::build()
				.header(ContentType::new("audio", "mpeg"))
				.header(ContentDisposition {
					disposition: DispositionType::Inline,
					parameters:  vec![DispositionParam::Filename(
						Charset::Ext("UTF-8".into()),
						None,
						name.into_bytes(),
					)],
				})
				.sized_body(std::io::Cursor::new(data))
				.finalize(),
			AudioFileResponse::NotFound => Response::build().status(rocket::http::Status::NotFound).finalize(),
		};

		Ok(response)
	}
}

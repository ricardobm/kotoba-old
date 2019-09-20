//! Entry point for japanese word and kanji queries.

use std::collections::HashMap;
use std::sync::mpsc::Sender;

use itertools::*;
use slog::Logger;

pub mod db;
pub mod import;

pub use self::db::search::*;

use kana::{is_kanji, to_romaji};

//
// Japanese pronunciation audio support
//

use audio::{AudioDataResult, AudioLoader, AudioQuery, AudioSource, SourceId};

pub fn new_audio_loader(base_path: &std::path::Path) -> AudioLoader<JapaneseAudioQuery> {
	AudioLoader::new(
		base_path,
		vec![
			Box::new(JishoSource {}),
			Box::new(JapanesePodSource {}),
			Box::new(LanguagePodSource {}),
			Box::new(ForvoSource {}),
		],
	)
}

pub trait JapaneseAudioSource: AudioSource<JapaneseAudioQuery> {}

/// Query arguments to load Japanese pronunciation audio.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct JapaneseAudioQuery {
	/// The main expression to lookup.
	pub expression: String,
	/// The reading for the expression to lookup.
	pub reading: String,
}

impl AudioQuery for JapaneseAudioQuery {
	fn query_hash(&self) -> String {
		let query_hash = format!("{}\n{}", self.expression, self.reading);
		let query_hash = crate::util::sha256(query_hash.as_bytes()).unwrap();
		query_hash
	}

	fn query_info(&self) -> String {
		format!("{} / {}", self.expression, self.reading)
	}
}

//
// Japanese audio sources
//

mod audio_helper;
mod forvo;
mod japanese_pod;
mod jisho;

use self::audio_helper::AudioSink;

struct JishoSource {}

impl JapaneseAudioSource for JishoSource {}

impl AudioSource<JapaneseAudioQuery> for JishoSource {
	fn copy(&self) -> Box<dyn AudioSource<JapaneseAudioQuery>> {
		Box::new(JishoSource {})
	}

	fn id(&self) -> &'static str {
		"jisho"
	}

	fn name(&self) -> &'static str {
		"Jisho"
	}

	fn load(&self, query: JapaneseAudioQuery, log: Logger, id: SourceId, sink: Sender<(SourceId, AudioDataResult)>) {
		let sink = AudioSink::new(log, id, sink);
		jisho::load_pronunciations(sink, &query.expression, &query.reading);
	}
}

struct JapanesePodSource {}

impl JapaneseAudioSource for JapanesePodSource {}

impl AudioSource<JapaneseAudioQuery> for JapanesePodSource {
	fn copy(&self) -> Box<dyn AudioSource<JapaneseAudioQuery>> {
		Box::new(JapanesePodSource {})
	}

	fn id(&self) -> &'static str {
		// spell-checker: disable
		"jpod"
		// spell-checker: enable
	}

	fn name(&self) -> &'static str {
		"JapanesePod101.com"
	}

	fn load(&self, query: JapaneseAudioQuery, log: Logger, id: SourceId, sink: Sender<(SourceId, AudioDataResult)>) {
		let sink = AudioSink::new(log, id, sink);
		japanese_pod::load_dictionary_pronunciations(sink, &query.expression, &query.reading);
	}
}

struct LanguagePodSource {}

impl JapaneseAudioSource for LanguagePodSource {}

impl AudioSource<JapaneseAudioQuery> for LanguagePodSource {
	fn copy(&self) -> Box<dyn AudioSource<JapaneseAudioQuery>> {
		Box::new(LanguagePodSource {})
	}

	fn id(&self) -> &'static str {
		// spell-checker: disable
		"langpod"
		// spell-checker: enable
	}

	fn name(&self) -> &'static str {
		"LanguagePod101.com"
	}

	fn load(&self, query: JapaneseAudioQuery, log: Logger, id: SourceId, sink: Sender<(SourceId, AudioDataResult)>) {
		let sink = AudioSink::new(log, id, sink);
		japanese_pod::load_pronunciation(sink, &query.expression, &query.reading);
	}
}

struct ForvoSource {}

impl JapaneseAudioSource for ForvoSource {}

impl AudioSource<JapaneseAudioQuery> for ForvoSource {
	fn copy(&self) -> Box<dyn AudioSource<JapaneseAudioQuery>> {
		Box::new(ForvoSource {})
	}

	fn id(&self) -> &'static str {
		// spell-checker: disable
		"forvo"
		// spell-checker: enable
	}

	fn name(&self) -> &'static str {
		"Forvo"
	}

	fn load(&self, query: JapaneseAudioQuery, log: Logger, id: SourceId, sink: Sender<(SourceId, AudioDataResult)>) {
		let sink = AudioSink::new(log, id, sink);
		forvo::load_pronunciations(sink, &query.expression, &query.reading);
	}
}

//
// Search
//

/// Search options.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchArgs {
	/// Main query string.
	pub query: String,

	/// If true, also returns kanjis from the query.
	#[serde(default)]
	pub with_kanji: bool,

	/// Search options.
	#[serde(default)]
	pub options: SearchOptions,
}

impl Default for SearchArgs {
	fn default() -> SearchArgs {
		SearchArgs {
			query:      String::new(),
			with_kanji: false,
			options:    Default::default(),
		}
	}
}

/// Japanese dictionary implementation.
pub struct Dictionary {
	db: db::Root,
}

impl Dictionary {
	pub fn new(db: db::Root) -> Dictionary {
		Dictionary { db }
	}

	pub fn get_db(&self) -> &db::Root {
		&self.db
	}

	/// Query the dictionary.
	pub fn query(&self, logger: &Logger, args: &SearchArgs) -> QueryResult {
		let start = std::time::Instant::now();

		let input = args.query.as_str();
		let romaji = to_romaji(input);

		let (terms, total) = self.db.search_terms(logger, input, &args.options);

		let kanji = if args.with_kanji {
			let kanji = input.chars().filter(|&x| is_kanji(x)).unique();
			let kanji = self.db.search_kanji(logger, kanji);
			Some(kanji)
		} else {
			None
		};

		let elapsed = start.elapsed().as_secs_f64();

		let mut tag_map = HashMap::new();

		let mut push_tag = |id: db::TagId| {
			if !tag_map.contains_key(&id) {
				let tag = self.db.get_tag(id);
				tag_map.insert(id, tag);
			}
		};

		for it in &terms {
			for &id in &it.tags {
				push_tag(id);
			}

			for definition in &it.definition {
				for &id in &definition.tags {
					push_tag(id);
				}
			}
		}

		if let Some(kanji) = &kanji {
			for it in kanji {
				for &id in &it.tags {
					push_tag(id);
				}
				for &id in it.stats.keys() {
					push_tag(id);
				}
			}
		}

		QueryResult {
			total:   total,
			elapsed: elapsed,
			query:   String::from(input),
			reading: romaji,
			terms:   terms,
			kanji:   kanji,
			tags:    tag_map,
			sources: self.db.sources.clone(),
			args:    args.clone(),
		}
	}

	pub fn db_map(&self) -> DbMap {
		let mut out = DbMap { sources: Vec::new() };
		for (src_index, it) in self.db.sources.iter().enumerate() {
			let src = DbSource {
				info: it.clone(),
				tags: self
					.db
					.tags
					.iter()
					.filter(|t| t.source.0 == src_index)
					.cloned()
					.collect(),
			};
			out.sources.push(src);
		}
		out
	}
}

/// Root for a dictionary query.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryResult {
	/// Total number of entries returned by the query, ignoring limit and offset.
	pub total: usize,

	/// Elapsed time in seconds.
	pub elapsed: f64,

	/// Input query.
	pub query: String,

	/// Input query reading.
	pub reading: String,

	/// List of terms returned by the query.
	pub terms: Vec<db::TermRow>,

	/// List of kanjis, if [SearchOptions::with_kanji] is true.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub kanji: Option<Vec<db::KanjiRow>>,

	/// List of tags.
	pub tags: HashMap<db::TagId, db::TagRow>,

	/// List of sources for the dictionary data.
	pub sources: Vec<db::SourceRow>,

	/// Arguments used in the search.
	pub args: SearchArgs,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DbMap {
	pub sources: Vec<DbSource>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DbSource {
	pub info: db::SourceRow,
	pub tags: Vec<db::TagRow>,
}

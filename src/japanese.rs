//! Entry point for japanese word and kanji queries.

use std::collections::HashMap;

use itertools::*;
use kana::{is_kanji, to_romaji};

use super::db;
use super::db::Search;

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
	pub options: db::SearchOptions,
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
	pub fn query(&self, args: &SearchArgs) -> QueryResult {
		let start = std::time::Instant::now();

		let input = args.query.as_str();
		let romaji = to_romaji(input);

		let (terms, total) = self.db.search_terms(input, &args.options);

		let kanji = if args.with_kanji {
			let kanji = input.chars().filter(|&x| is_kanji(x)).unique();
			let kanji = self.db.search_kanji(kanji);
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

//! Entry point for japanese word and kanji queries.

use std::collections::HashMap;

use itertools::*;
use kana::{is_kanji, to_romaji};

use super::db;
pub use db::{InputString, Search, SearchMode};

/// Search options.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SearchOptions {
	/// Search mode.
	#[serde(default)]
	pub mode: SearchMode,

	/// If true, will also look for near matches.
	#[serde(default)]
	pub fuzzy: bool,

	/// Skip this number of terms from beginning of the results.
	#[serde(default)]
	pub offset: usize,

	/// Limit of terms to return.
	#[serde(default)]
	pub limit: usize,

	/// If true, search for kanjis from the query.
	#[serde(default)]
	pub with_kanji: bool,
}

impl Default for SearchOptions {
	fn default() -> SearchOptions {
		SearchOptions {
			mode:       SearchMode::Contains,
			fuzzy:      false,
			offset:     0,
			limit:      0,
			with_kanji: false,
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

	/// Query the dictionary.
	pub fn query_with_options<'a, S: InputString<'a>>(&self, input: S, options: SearchOptions) -> QueryResult {
		let start = std::time::Instant::now();

		let query = String::from(input.into());
		let romaji = to_romaji(&query);

		let terms = self.db.search_terms(&query, &romaji, options.mode, options.fuzzy);

		let kanji = if options.with_kanji {
			let kanji = query.chars().filter(|&x| is_kanji(x)).unique();
			let kanji = self.db.search_kanji(kanji);
			Some(kanji)
		} else {
			None
		};

		let total = terms.len();
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

		let terms = if options.offset > 0 {
			terms.into_iter().skip(options.offset).collect()
		} else {
			terms
		};

		let terms = if options.limit > 0 {
			terms.into_iter().take(options.limit).collect()
		} else {
			terms
		};

		QueryResult {
			total:   total,
			elapsed: elapsed,
			query:   String::from(&query),
			reading: to_romaji(query),
			terms:   terms,
			kanji:   kanji,
			tags:    tag_map,
			sources: self.db.sources.clone(),
			options: options,
		}
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

	/// Options used in the search.
	pub options: SearchOptions,
}

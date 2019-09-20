//! Dictionary search service.

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use slog::Logger;

use itertools::*;

use super::db;
use super::Search;
use super::SearchOptions;

use kana::{is_kanji, normalize_search_string, to_romaji};

/// Search options.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchArgs {
	/// An optional ID that will be returned with the results.
	#[serde(default)]
	pub id: usize,

	/// Main query string.
	pub query: String,

	/// If true, also returns kanjis from the query and results.
	#[serde(default)]
	pub with_kanji: bool,

	/// Search options.
	#[serde(default)]
	pub options: SearchOptions,
}

impl Default for SearchArgs {
	fn default() -> SearchArgs {
		SearchArgs {
			id:         0,
			query:      String::new(),
			with_kanji: false,
			options:    Default::default(),
		}
	}
}

/// Result for a dictionary query.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response {
	/// Optional ID
	pub id: usize,

	/// Total number of entries returned by the query, ignoring limit and offset.
	pub total: usize,

	/// Elapsed time in seconds searching.
	pub elapsed: f64,

	/// Input query, normalized.
	pub expression: String,

	/// Romaji reading for the input query.
	pub reading: String,

	/// List of terms returned by the query.
	pub terms: Vec<Term>,

	/// List of kanjis, if [SearchOptions::kanji] is true.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub kanji: Option<Vec<Kanji>>,

	/// Tags used in the results.
	pub tags: HashMap<String, Tag>,

	/// List of sources used in the results.
	pub sources: Vec<Source>,

	/// Arguments used in the search.
	pub args: SearchArgs,
}

/// Kanji result.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Kanji {
	pub character: char,
	pub onyomi:    Vec<String>,
	pub kunyomi:   Vec<String>,
	pub tags:      Vec<String>,
	pub meanings:  Vec<String>,
	pub stats:     HashMap<String, String>,
	pub frequency: Option<u64>,
}

impl Kanji {
	pub fn from_row(kanji: db::KanjiRow, tags: &HashMap<db::TagId, db::TagRow>) -> Kanji {
		Kanji {
			character: kanji.character,
			onyomi:    kanji.onyomi,
			kunyomi:   kanji.kunyomi,
			tags:      kanji.tags.into_iter().map(|x| tags[&x].name.clone()).collect(),
			meanings:  kanji.meanings,
			stats:     kanji
				.stats
				.into_iter()
				.map(|(k, v)| (tags[&k].name.clone(), v))
				.collect(),
			frequency: kanji.frequency,
		}
	}
}

/// Japanese term result.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Term {
	pub expression: String,
	pub reading:    String,
	pub romaji:     String,
	pub definition: Vec<Definition>,
	pub sources:    Vec<String>,
	pub forms:      Vec<Form>,
	pub tags:       Vec<String>,
	pub frequency:  Option<u64>,
}

impl Term {
	pub fn from_row(term: db::TermRow, tags: &HashMap<db::TagId, db::TagRow>, sources: &Vec<db::SourceRow>) -> Term {
		Term {
			expression: term.expression,
			reading:    term.reading,
			romaji:     term.romaji,
			definition: term
				.definition
				.into_iter()
				.map(|x| Definition::from_row(x, tags))
				.collect(),
			sources:    term
				.source
				.into_iter()
				.map(|db::SourceIndex(x)| sources[x].name.clone())
				.collect(),
			forms:      term
				.forms
				.into_iter()
				.map(|x| Form {
					expression: x.expression,
					reading:    x.reading,
					romaji:     x.romaji,
					frequency:  x.frequency,
				})
				.collect(),
			tags:       term.tags.into_iter().map(|x| tags[&x].name.clone()).collect(),
			frequency:  term.frequency,
		}
	}
}

/// Tag result.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tag {
	pub name:        String,
	pub category:    String,
	pub description: String,
	pub source:      String,
}

impl Tag {
	pub fn from_row(tag: db::TagRow, sources: &Vec<db::SourceRow>) -> Tag {
		let db::SourceIndex(source_index) = tag.source;
		Tag {
			name:        tag.name,
			category:    tag.category,
			description: tag.description,
			source:      sources[source_index].name.clone(),
		}
	}
}

/// Source result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Source {
	pub name:     String,
	pub revision: String,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub tags: Option<HashMap<String, Tag>>,
}

impl Source {
	pub fn from_row(row: &db::SourceRow) -> Source {
		Source {
			name:     row.name.clone(),
			revision: row.revision.clone(),
			tags:     None,
		}
	}
}

/// Definition for a [Term].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Definition {
	pub text: Vec<String>,
	pub info: Vec<String>,
	pub tags: Vec<String>,
	pub link: Vec<Link>,
}

impl Definition {
	pub fn from_row(def: db::DefinitionRow, tags: &HashMap<db::TagId, db::TagRow>) -> Definition {
		Definition {
			text: def.text,
			info: def.info,
			tags: def.tags.into_iter().map(|x| tags[&x].name.clone()).collect(),
			link: def
				.link
				.into_iter()
				.map(|x| Link {
					uri:   x.uri,
					title: x.title,
				})
				.collect(),
		}
	}
}

/// Additional form for a [Term].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Form {
	pub expression: String,
	pub reading:    String,
	pub romaji:     String,
	pub frequency:  Option<u64>,
}

/// Links for a [Definition].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Link {
	pub uri:   String,
	pub title: String,
}

/// Japanese dictionary search service.
pub struct Dictionary {
	db: Arc<db::Root>,
}

impl Dictionary {
	pub fn new(db: Arc<db::Root>) -> Dictionary {
		Dictionary { db }
	}

	pub fn query(&self, log: &Logger, args: SearchArgs) -> Response {
		time!(t_query);

		// Normalize the expression and translate to romaji
		let expression = normalize_search_string(&args.query, true);
		let reading = to_romaji(&expression);

		// Search for dictionary terms
		let (terms, total) = self.db.search_terms(log, &args.query, &args.options);

		// Search for kanji in the query and results:

		// First we search for kanji direct from the query term...
		let mut kanji_map = HashSet::new();
		let kanji = if args.with_kanji {
			let kanji = expression.chars().filter(|&x| is_kanji(x)).unique().collect::<Vec<_>>();
			for chr in kanji.iter() {
				kanji_map.insert(*chr);
			}
			Some(self.db.search_kanji(log, kanji))
		} else {
			None
		};

		// Next we append kanji from the results themselves...
		let kanji = if let Some(mut result) = kanji {
			let mut to_search = Vec::new();
			for it in terms.iter() {
				let kanji = std::iter::once(&it.expression)
					.chain(it.forms.iter().map(|x| &x.expression))
					.map(|x| x.chars())
					.flatten()
					.filter(|&x| is_kanji(x));
				for chr in kanji {
					if kanji_map.insert(chr) {
						to_search.push(chr);
					}
				}
			}
			let mut extra_kanji = self.db.search_kanji(log, to_search);
			extra_kanji.sort_by(|a, b| b.frequency.cmp(&a.frequency));
			result.append(&mut extra_kanji);
			Some(result)
		} else {
			None
		};

		//
		// Build the results:
		//

		let elapsed = t_query.elapsed().as_secs_f64();

		// Collect all tags:

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

		let mut response = Response {
			id:         args.id,
			total:      total,
			elapsed:    elapsed,
			expression: expression.to_string(),
			reading:    reading.to_string(),
			terms:      Default::default(),
			kanji:      None,
			tags:       Default::default(),
			sources:    Default::default(),
			args:       args.clone(),
		};

		let sources = &self.db.sources;

		for it in terms {
			response.terms.push(Term::from_row(it, &tag_map, sources));
		}

		if let Some(kanji) = kanji {
			response.kanji = Some(kanji.into_iter().map(|x| Kanji::from_row(x, &tag_map)).collect());
		}

		for (_, tag) in tag_map {
			response.tags.insert(tag.name.clone(), Tag::from_row(tag, sources));
		}

		for src in sources.iter() {
			response.sources.push(Source::from_row(src));
		}

		response
	}

	pub fn tags(&self) -> Vec<Source> {
		let sources = &self.db.sources;
		let mut response = Vec::new();
		for (src_index, src) in sources.iter().enumerate() {
			let mut source = Source::from_row(src);
			let tags = self.db.tags.iter().filter(|t| t.source.0 == src_index).cloned();

			let mut tag_map = HashMap::new();
			for tag in tags {
				tag_map.insert(tag.name.clone(), Tag::from_row(tag, sources));
			}
			source.tags = Some(tag_map);
			response.push(Source::from_row(src));
		}
		response
	}
}

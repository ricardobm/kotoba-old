//! Entry point for japanese word and kanji queries.

use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::HashSet;

pub trait InputString<'a>: Into<Cow<'a, str>> {}

impl<'a, T> InputString<'a> for T where T: Into<Cow<'a, str>> {}

/// Available search modes for terms.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum SearchMode {
	/// Search for exact word.
	Is,
	/// Search words starting with the query.
	Prefix,
	/// Search words ending with the query.
	Suffix,
	/// Search words containing the query.
	Contains,
}

impl Default for SearchMode {
	fn default() -> SearchMode {
		SearchMode::Contains
	}
}

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

use super::dict;

/// Japanese dictionary implementation.
pub struct Dictionary {
	dict: dict::Dict,
}

impl Dictionary {
	pub fn new(dict: dict::Dict) -> Dictionary {
		Dictionary { dict }
	}

	/// Query the dictionary.
	pub fn query_with_options<'a, S: InputString<'a>>(&self, input: S, options: SearchOptions) -> QueryResult {
		let start = std::time::Instant::now();
		let query = String::from(input.into());
		let results = self.dict.search(
			&query,
			match options.mode {
				SearchMode::Is => dict::SearchMode::Exact,
				SearchMode::Prefix => dict::SearchMode::Prefix,
				SearchMode::Suffix => dict::SearchMode::Suffix,
				SearchMode::Contains => dict::SearchMode::Contains,
			},
		);

		let mut tag_map = TagMap::new();
		let mut terms = Vec::new();
		for it in results {
			let expressions = it.expressions();
			let readings = it.readings();
			let mut result = Term {
				expression: String::from(expressions[0]),
				reading:    String::from(readings[0]),
				origin:     String::from(it.origin()),
				forms:      Vec::new(),
				definition: Vec::new(),
				tags:       HashSet::new(),
				frequency:  None,
			};

			for tag in it.tags() {
				let id = tag_map.to_tag_id(tag);
				result.tags.insert(id);
			}

			for (i, expr) in expressions.into_iter().enumerate().skip(1) {
				let expr = String::from(expr);
				let reading = String::from(readings[i]);
				result.forms.push(Form(expr, reading));
			}
			for it in it.definition() {
				let def = Definition {
					text: it.glossary().into_iter().map(String::from).collect(),
					info: it.info().into_iter().map(String::from).collect(),
					tags: HashSet::new(),
					link: Vec::new(),
				};

				for tag in it.tags() {
					let id = tag_map.to_tag_id(tag);
					result.tags.insert(id);
				}

				result.definition.push(def);
			}
			terms.push(result);
		}

		let total = terms.len();
		let elapsed = start.elapsed().as_secs_f64();

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
			kanjis:  None,
			tags:    tag_map.tags(),
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
	pub terms: Vec<Term>,

	/// List of kanjis, if [SearchOptions::with_kanji] is true.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub kanjis: Option<Vec<Kanji>>,

	/// List of tags.
	pub tags: HashMap<TagId, Tag>,

	/// Options used in the search.
	pub options: SearchOptions,
}

/// Identifier for a tag.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TagId(String);

/// Entry for a kanji in the dictionary.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Kanji {
	/// Kanji character.
	pub character: char,

	/// Onyomi (chinese) readings for the Kanji.
	pub onyomi: Vec<String>,

	/// Kunyomi (japanese) readings for the Kanji.
	pub kunyomi: Vec<String>,

	/// ID of the tags that apply to this kanji.
	pub tags: HashSet<TagId>,

	/// English meanings for the kanji.
	pub meanings: Vec<String>,

	/// Additional kanji information. The key meanings are detailed as tags.
	pub stats: HashMap<TagId, String>,

	/// Frequency information for this kanji, if available.
	pub frequency: Option<u32>,
}

/// Main entry for a word in the dictionary.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Term {
	/// Main expression.
	pub expression: String,

	/// Kana reading for the main expression.
	pub reading: String,

	/// Definitions for this term.
	pub definition: Vec<Definition>,

	/// Description of the origin for this entry.
	pub origin: String,

	/// Additional forms for the term, if any.
	pub forms: Vec<Form>,

	/// ID of the tags that apply to this term.
	pub tags: HashSet<TagId>,

	/// Frequency information for this term, if available.
	pub frequency: Option<u32>,
}

/// English definition for a [Term].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Definition {
	/// Definition text entries.
	pub text: Vec<String>,

	/// Additional information to append after this entry text.
	pub info: Vec<String>,

	/// ID of the tags that apply to this definition.
	pub tags: HashSet<TagId>,

	/// Resources linked to this definition.
	pub link: Vec<Link>,
}

/// Additional `(expression, reading)` for a [Term].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Form(String, String);

/// Linked resource in the form `(URI, title)`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Link(String, String);

/// Information for a tag.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tag {
	/// The ID for this tag referenced by a [Entry] or [Definition].
	pub id: TagId,

	/// Short display name for this tag.
	pub name: String,

	/// Category name for this tag.
	pub category: String,

	/// Longer display description for this tag.
	pub description: String,

	/// Order value for this tag (lesser values sorted first).
	pub order: i32,
}

pub fn to_romaji<'a, S>(input: S) -> String
where
	S: InputString<'a> + std::fmt::Display,
{
	let mut text = String::from(input.into());

	// The kana library completely barfs on "っっ", so replace it by "っ".
	while text.contains("っっ") {
		text = text.replace("っっ", "っ");
	}
	while text.contains("ッッ") {
		text = text.replace("ッッ", "ッ");
	}

	let result = std::panic::catch_unwind(|| wana_kana::to_romaji::to_romaji(text.as_str()));
	match result {
		Ok(value) => value,
		Err(_) => panic!(format!("\n!\n! FAILED: to_romaji({})\n!\n", text)),
	}
}

struct TagMap {
	tags:     HashMap<TagId, Tag>,
	by_index: HashMap<usize, TagId>,
	by_id:    HashMap<TagId, usize>,
}

impl TagMap {
	fn new() -> TagMap {
		TagMap {
			tags:     HashMap::new(),
			by_index: HashMap::new(),
			by_id:    HashMap::new(),
		}
	}

	fn tags(self) -> HashMap<TagId, Tag> {
		self.tags
	}

	fn to_tag_id<'a>(&mut self, tag: dict::Tag<'a>) -> TagId {
		let index = tag.index();
		if let Some(id) = self.by_index.get(&index) {
			id.clone()
		} else {
			let name = tag.name();
			let mut counter = 1;
			let new_id = loop {
				let id = TagId(format!("{}-{}", name, counter));
				match self.by_id.entry(id.clone()) {
					Entry::Occupied(_) => {
						counter += 1;
						continue;
					}
					Entry::Vacant(entry) => {
						entry.insert(index);
						break id;
					}
				}
			};
			self.by_index.insert(index, new_id.clone());
			self.tags.insert(
				new_id.clone(),
				Tag {
					id:          new_id.clone(),
					name:        String::from(tag.name()),
					description: String::from(tag.description()),
					category:    String::from(tag.category()),
					order:       tag.order(),
				},
			);

			new_id
		}
	}
}

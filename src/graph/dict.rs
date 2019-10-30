use std::collections::HashMap;
use std::collections::HashSet;

use itertools::Itertools;
use regex::{escape, Regex};
use unicase::UniCase;

use super::Context;
use japanese::db;
use japanese::db::search::Search;
use kana::{normalize_search_string, to_romaji};

/// Options for a dictionary term search.
#[derive(juniper::GraphQLInputObject)]
pub struct SearchOptions {
	/// Search mode.
	mode: Option<db::search::SearchMode>,
	/// Offset to return in the results.
	offset: Option<i32>,
	/// Limit of terms to return in the results.
	limit: Option<i32>,
}

/// Japanese dictionary queries.
pub struct DictQuery;

#[juniper::object(Context = Context)]
impl DictQuery {
	/// Lookup terms in the dictionary.
	fn search(context: &Context, filter: Option<String>, options: Option<SearchOptions>) -> TermResult {
		let db = context.app.db();
		let (mode, offset, limit) = if let Some(options) = options {
			let mode = options.mode.unwrap_or_default();
			(mode, options.offset, options.limit)
		} else {
			(Default::default(), None, Some(1000))
		};
		let options = db::search::SearchOptions {
			mode,
			offset: match offset {
				Some(v) if v >= 0 => v as usize,
				_ => 0,
			},
			limit: match limit {
				Some(v) if v >= 0 => v as usize,
				_ => 0,
			},
			fuzzy: false,
		};

		time!(t_query);
		let (terms, total, expression, reading) = if let Some(filter) = filter {
			// Normalize the expression and translate to romaji
			let expression = normalize_search_string(&filter, true);
			let reading = to_romaji(&expression);

			// Search for dictionary terms
			let (terms, total) = db.search_terms(&context.log, &filter, &options);
			(terms, total, expression, reading)
		} else {
			let terms = db.terms.iter().cloned().enumerate().skip(options.offset);
			let terms = if options.limit > 0 {
				terms.take(options.limit).collect::<Vec<_>>()
			} else {
				terms.collect::<Vec<_>>()
			};
			let total = db.terms.len();
			(terms, total, String::new(), String::new())
		};

		let elapsed = t_query.elapsed().as_secs_f64();

		// Collect all tags:

		let mut tag_map = HashMap::new();

		let mut push_tag = |id: db::TagId| {
			if !tag_map.contains_key(&id) {
				let tag = db.get_tag(id);
				tag_map.insert(id, tag);
			}
		};

		for (_, it) in &terms {
			for id in it.tag_ids() {
				push_tag(id);
			}
		}

		let mut source_set = HashSet::new();
		for (_, it) in &terms {
			for &db::SourceIndex(index) in &it.source {
				source_set.insert(index);
			}
		}

		let mut response = TermResult {
			total:      total as i32,
			elapsed:    elapsed,
			expression: expression,
			reading:    reading,
			terms:      terms.into_iter().map(|(index, row)| Term { index, row }).collect(),
			tags:       to_tags(tag_map.values(), None),
			sources:    Default::default(),
			options:    SearchOptions {
				mode:   Some(mode),
				offset: offset,
				limit:  limit,
			},
		};

		for (index, src) in db.sources.iter().enumerate() {
			if source_set.contains(&index) {
				response.sources.push(Source { row: src.clone() })
			}
		}

		response
	}

	/// All dictionary sources.
	fn sources(context: &Context) -> Vec<Source> {
		let db = context.app.db();
		db.sources.iter().map(|it| Source { row: it.clone() }).collect()
	}

	/// Dictionary source by exact name.
	fn source_by_name(context: &Context, name: String) -> Option<Source> {
		let db = context.app.db();
		db.sources
			.iter()
			.filter(|it| it.name == name)
			.map(|it| Source { row: it.clone() })
			.next()
	}

	/// Dictionary tags.
	fn tags(context: &Context, filter: Option<String>) -> Vec<Tag> {
		let db = context.app.db();
		to_tags(db.tags.iter(), filter)
	}
}

/// Result of a term search.
pub struct TermResult {
	total:      i32,
	elapsed:    f64,
	expression: String,
	reading:    String,
	terms:      Vec<Term>,
	tags:       Vec<Tag>,
	sources:    Vec<Source>,
	options:    SearchOptions,
}

#[juniper::object(Context = Context)]
impl TermResult {
	/// Total number of entries in the result, regardless of the specified limit
	/// or offset.
	fn total(&self) -> i32 {
		self.total
	}

	/// Mode that was used to search the results.
	fn mode(&self) -> db::search::SearchMode {
		self.options.mode.unwrap_or_default()
	}

	/// Limit defined by the search options.
	fn limit(&self) -> Option<i32> {
		self.options.limit
	}

	/// Offset defined by the search options.
	fn offset(&self) -> Option<i32> {
		self.options.offset
	}

	/// Number of seconds elapsed in the search.
	fn elapsed(&self) -> f64 {
		self.elapsed
	}

	/// Normalized input expression.
	fn expression(&self) -> &String {
		&self.expression
	}

	/// Romaji reading for the input expression.
	fn reading(&self) -> &String {
		&self.reading
	}

	/// Terms in the result.
	fn terms(&self) -> &Vec<Term> {
		&self.terms
	}

	/// All tags in the result.
	fn tags(&self) -> &Vec<Tag> {
		&self.tags
	}

	/// Sources in the result.
	fn sources(&self) -> &Vec<Source> {
		&self.sources
	}
}

/// Main entry for a word in the dictionary.
pub struct Term {
	index: usize,
	row:   db::TermRow,
}

#[juniper::object(Context = Context)]
impl Term {
	/// Unique ID for this term.
	fn id(&self) -> i32 {
		self.index as i32
	}

	/// Main expression.
	fn expression(&self) -> &String {
		&self.row.expression
	}

	/// Kana reading for the main expression.
	fn reading(&self) -> &String {
		&self.row.reading
	}

	/// Kana reading as romaji.
	fn romaji(&self) -> &String {
		&self.row.romaji
	}

	/// Definitions for this term.
	fn definition(&self) -> Vec<Definition> {
		self.row
			.definition
			.iter()
			.map(|x| Definition { row: x.clone() })
			.collect()
	}

	/// Sources for this entry.
	fn source(&self, context: &Context) -> Vec<Source> {
		let db = context.app.db();
		self.row
			.source
			.iter()
			.map(|&db::SourceIndex(index)| Source {
				row: db.sources[index].clone(),
			})
			.collect()
	}

	/// Additional forms for the term, if any.
	fn forms(&self) -> Vec<Form> {
		self.row.forms.iter().map(|x| Form { row: x.clone() }).collect()
	}

	/// Tags for the term.
	fn tags(&self, context: &Context) -> Vec<Tag> {
		tags_from_row(context, &self.row.tags)
	}

	/// Frequency information for this term, when available.
	fn frequency(&self) -> Option<i32> {
		self.row.frequency.map(|x| x as i32)
	}

	/// Numeric score of this entry to be used as a tie breaker for sorting.
	///
	/// Higher values should appear first. Note that using this with entries
	/// from different sources is meaningless.
	fn score(&self) -> i32 {
		self.row.score
	}
}

/// English definition for term.
pub struct Definition {
	row: db::DefinitionRow,
}

#[juniper::object(Context = Context)]
impl Definition {
	/// Definition text entries.
	pub fn text() -> &Vec<String> {
		&self.row.text
	}

	/// Additional information to append after this entry text.
	pub fn info() -> &Vec<String> {
		&self.row.info
	}

	/// Tags for the definition.
	fn tags(&self, context: &Context) -> Vec<Tag> {
		tags_from_row(context, &self.row.tags)
	}
}

pub struct Form {
	row: db::FormRow,
}

#[juniper::object(Context = Context)]
impl Form {
	/// Expression for this form.
	fn expression(&self) -> &String {
		&self.row.expression
	}

	/// Kana reading for the expression.
	fn reading(&self) -> &String {
		&self.row.reading
	}

	/// Kana reading as romaji.
	fn romaji(&self) -> &String {
		&self.row.romaji
	}

	/// Frequency information for this form, when available.
	fn frequency(&self) -> Option<i32> {
		self.row.frequency.map(|x| x as i32)
	}
}

/// Tag for a dictionary term or kanji.
///
/// For Kanji this also describes the stats keys.
pub struct Tag {
	row: db::TagRow,
}

#[juniper::object(Context = Context)]
impl Tag {
	/// Short display name for this tag.
	fn name() -> &String {
		&self.row.name
	}

	/// Category name for this tag.
	fn category() -> &String {
		&self.row.category
	}

	/// Longer display description for this tag.
	fn description() -> &String {
		&self.row.description
	}

	/// Order value for this tag (lesser values sorted first).
	fn order() -> i32 {
		self.row.order
	}

	/// Tag source dictionary.
	fn source(context: &Context) -> Source {
		let db = context.app.db();
		let db::SourceIndex(index) = self.row.source;
		Source {
			row: db.sources[index].clone(),
		}
	}
}

/// Source for dictionary definitions.
pub struct Source {
	row: db::SourceRow,
}

#[juniper::object(Context = Context)]
impl Source {
	/// Human readable name for this source.
	fn name(&self) -> &String {
		&self.row.name
	}

	/// Source revision.
	fn revision(&self) -> &String {
		&self.row.revision
	}

	fn tags(&self, context: &Context, filter: Option<String>) -> Vec<Tag> {
		let db = context.app.db();
		let index = db
			.sources
			.iter()
			.enumerate()
			.filter(|x| x.1.name == self.row.name)
			.next();
		if let Some((index, _)) = index {
			let index = db::SourceIndex(index);
			to_tags(
				db.tags.iter().filter(|x| if x.source == index { true } else { false }),
				filter,
			)
		} else {
			Default::default()
		}
	}
}

fn tags_from_row(context: &Context, tags: &HashSet<db::tables::TagId>) -> Vec<Tag> {
	let db = context.app.db();
	to_tags(tags.iter().map(|&db::tables::TagId(index)| &db.tags[index]), None)
}

fn to_tags<'a, T>(tags: T, filter: Option<String>) -> Vec<Tag>
where
	T: IntoIterator<Item = &'a db::TagRow>,
{
	let filter = filter.map(|x| Regex::new(&escape(x.as_str())).unwrap());
	tags.into_iter()
		.filter(|x| {
			if let Some(filter) = &filter {
				filter.is_match(&x.name)
			} else {
				true
			}
		})
		.sorted_by(|a, b| {
			let a = UniCase::new(&a.name);
			let b = UniCase::new(&b.name);
			a.cmp(&b)
		})
		.map(|x| Tag { row: x.clone() })
		.collect()
}

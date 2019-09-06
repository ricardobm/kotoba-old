use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;

use super::tables::*;

/// Wrapper trait for a generic input string.
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

/// Trait for doing database searches.
pub trait Search {
	fn search_terms<'a, 'b, S1, S2>(&self, query: S1, romaji: S2, mode: SearchMode, fuzzy: bool) -> Vec<TermRow>
	where
		S1: InputString<'a>,
		S2: InputString<'b>;

	fn search_kanji<T>(&self, query: T) -> Vec<KanjiRow>
	where
		T: IntoIterator<Item = char>;
}

impl Search for Root {
	fn search_terms<'a, 'b, S1, S2>(&self, query: S1, _romaji: S2, mode: SearchMode, _fuzzy: bool) -> Vec<TermRow>
	where
		S1: InputString<'a>,
		S2: InputString<'b>,
	{
		let mut indexes: HashSet<usize> = HashSet::new();
		let query = query.into();
		for (key, val) in &self.mem_index.terms {
			let is_match = match mode {
				SearchMode::Is => key == query.as_ref(),
				SearchMode::Contains => key.contains(query.as_ref()),
				SearchMode::Prefix => key.starts_with(query.as_ref()),
				SearchMode::Suffix => key.ends_with(query.as_ref()),
			};
			if is_match {
				indexes.extend(val.iter());
			}
		}

		let mut out = Vec::new();
		for index in indexes {
			out.push(self.terms[index].clone());
		}

		out
	}

	fn search_kanji<T>(&self, query: T) -> Vec<KanjiRow>
	where
		T: IntoIterator<Item = char>,
	{
		let mut out = Vec::new();
		for it in query.into_iter() {
			if let Some(&index) = self.mem_index.kanji.get(&it) {
				out.push(self.kanji[index].clone());
			}
		}
		out
	}
}

//
// Indexes implementation
//

/// Memory (i.e. not persisted) index for a [Root] database.
pub struct MemoryIndex {
	/// Index of kanji in the database, by their character.
	pub kanji: HashMap<char, usize>,
	/// Index of terms in the database, by the expression and reading.
	pub terms: HashMap<String, HashSet<usize>>,
}

impl Default for MemoryIndex {
	fn default() -> MemoryIndex {
		MemoryIndex {
			kanji: HashMap::new(),
			terms: HashMap::new(),
		}
	}
}

impl MemoryIndex {
	pub fn new() -> MemoryIndex {
		MemoryIndex::default()
	}

	#[inline]
	fn index_term(&mut self, term: &String, index: usize) {
		self.terms
			.entry(term.to_owned())
			.and_modify(|e| {
				e.insert(index);
			})
			.or_insert_with(|| {
				let mut h = HashSet::new();
				h.insert(index);
				h
			});
	}
}

pub fn update_mem_index(db: &mut Root) {
	db.mem_index.kanji.clear();
	db.mem_index.terms.clear();

	for (i, it) in db.kanji.iter().enumerate() {
		db.mem_index.kanji.insert(it.character, i);
	}

	for (i, it) in db.terms.iter().enumerate() {
		db.mem_index.index_term(&it.expression, i);
		db.mem_index.index_term(&it.reading, i);
		for form in &it.forms {
			db.mem_index.index_term(&form.expression, i);
			db.mem_index.index_term(&form.reading, i);
		}
	}
}

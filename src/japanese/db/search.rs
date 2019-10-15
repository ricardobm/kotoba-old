use std::collections::HashSet;

use itertools::*;
use slog::Logger;

use super::tables::*;
use japanese;
use kana::*;

/// Available search modes for terms.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
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

impl SearchMode {
	/// Does this search mode allow for prefix search.
	fn includes_prefix(&self) -> bool {
		match self {
			SearchMode::Prefix | SearchMode::Contains => true,
			_ => false,
		}
	}

	/// Does this search mode allow for suffix search.
	fn includes_suffix(&self) -> bool {
		match self {
			SearchMode::Suffix | SearchMode::Contains => true,
			_ => false,
		}
	}
}

/// Search options.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchOptions {
	#[serde(default)]
	pub mode: SearchMode,

	#[serde(default)]
	pub fuzzy: bool,

	#[serde(default)]
	pub offset: usize,

	#[serde(default)]
	pub limit: usize,
}

impl Default for SearchOptions {
	fn default() -> SearchOptions {
		SearchOptions {
			mode:   Default::default(),
			fuzzy:  false,
			offset: 0,
			limit:  0,
		}
	}
}

pub struct WordSearch {
	/// Matched text.
	pub text: String,
	/// De-inflected term that matched the text.
	pub term: String,
	/// Terms for the match.
	pub list: Vec<(usize, TermRow)>,
	/// De-inflection transformations.
	pub info: Vec<&'static str>,
}

/// Trait for doing database searches.
pub trait Search {
	fn search_terms<S>(&self, log: &Logger, query: S, options: &SearchOptions) -> (Vec<(usize, TermRow)>, usize)
	where
		S: AsRef<str>;

	fn search_kanji<T>(&self, log: &Logger, query: T) -> Vec<KanjiRow>
	where
		T: IntoIterator<Item = char>;

	/// Best attempt to match the longest possible prefix of `query` to a single
	/// word in the database.
	///
	/// The best attempt includes matching de-inflected forms of the word.
	fn match_prefix<S>(&self, query: S) -> Option<WordSearch>
	where
		S: AsRef<str>;

	/// Search a single word in the database.
	///
	/// This is similar to [search_terms] with a [SearchMode::Is] without fuzzy
	/// mode, but supports searching for de-inflected forms.
	fn search_word<S>(&self, word: S, deinflect: bool) -> Option<WordSearch>
	where
		S: AsRef<str>;
}

/// Normalize the input, split it and filter out unsearchable characters.
///
/// Normalization occurs as following:
/// - The text is normalized to the Unicode NFC form and converted to lowercase.
/// - Katakana and Romaji are converted to Hiragana.
/// - Intraword punctuation chars and `ー` are removed.
/// - The Katakana `ー` is also removed.
/// - The result is split by punctuation and spaces.
/// - Non-kanji and non-kana characters are removed.
pub fn search_strings<S: AsRef<str>>(query: S) -> Vec<String> {
	let text = crate::kana::normalize_search_string(query, true);
	search_strings_normalized(text)
}

fn search_strings_normalized<S: AsRef<str>>(text: S) -> Vec<String> {
	let groups = text
		.as_ref()
		.chars()
		.filter(|&c| !intra_word_removable(c))
		.group_by(|&c| is_word_split(c));
	groups
		.into_iter()
		// Filter out group of split characters
		.filter(|it| !it.0)
		.map(|(_, e)| {
			// Filter out non-searchable characters
			e.filter(|&c| is_searchable(c)).collect::<String>()
		})
		// Filter out empty groups
		.filter(|it| it.len() > 0)
		.collect::<Vec<_>>()
}

/// Provides a first-pass broad key to index words for a `contains` type
/// search.
///
/// A key `(c0, c2)` will index all words that contain `c0` and `c2` anywhere in
/// the word, provided `c2` appears anywhere after `c1`.
///
/// For a single char index, `c2` will be `\0`.
#[derive(Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct SearchKey(char, char);

impl std::fmt::Display for SearchKey {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		if self.1 != '\0' {
			write!(f, "{}{}", self.0, self.1)
		} else {
			write!(f, "{}", self.0)
		}
	}
}

/// Iterate over all possible search keys for the key. Note that this will
/// not de-duplicate the output.
///
/// This assumes the key is already normalized.
#[allow(dead_code)]
pub fn search_keys<S: AsRef<str>>(key: S) -> SearchKeyIterator<S> {
	SearchKeyIterator {
		key: key,
		pos: 0,

		pair_char: None,
		pair_next: 0,
	}
}

pub struct SearchKeyIterator<S: AsRef<str>> {
	key: S,
	pos: usize,

	pair_char: Option<char>,
	pair_next: usize,
}

impl<S: AsRef<str>> std::iter::Iterator for SearchKeyIterator<S> {
	type Item = SearchKey;

	fn next(&mut self) -> Option<Self::Item> {
		let key = self.key.as_ref();

		// Continue a previous character key pair iteration
		if let Some(c1) = self.pair_char {
			let mut chars = Self::chars(key, self.pair_next);
			if let Some((_, c2)) = chars.next() {
				self.pair_next = Self::next_index(key, chars);
				return Some(SearchKey(c1, c2));
			} else {
				self.pair_char = None;
			}
		}

		if self.pos >= key.len() {
			return None;
		}

		// Find the next indexable char in the key string...
		let mut chars = Self::chars(key, self.pos);
		if let Some((_, chr)) = chars.next() {
			self.pos = Self::next_index(key, chars);
			if is_kanji(chr) {
				// if it is a kanji, use it by itself as a key. Most kanji
				// are uncommon enough to provide good enough filtering.
				Some(SearchKey(chr, '\0'))
			} else {
				// for kana characters we want to provide better filtering
				// than a single character would allow (since they are so
				// common), so we group characters two by two

				self.pair_char = Some(chr);
				self.pair_next = self.pos;

				// Still provide a single character key anyway. We use that
				// for search strings that have a single character.
				Some(SearchKey(chr, '\0'))
			}
		} else {
			None
		}
	}
}

impl<S: AsRef<str>> SearchKeyIterator<S> {
	#[inline]
	fn chars<'a>(key: &'a str, at: usize) -> impl 'a + Iterator<Item = (usize, char)> {
		key[at..]
			.char_indices()
			.filter(|(_, c)| is_searchable(*c))
			.map(move |(i, c)| (i + at, c))
	}

	#[inline]
	fn next_index<T>(key: &str, mut iter: T) -> usize
	where
		T: Iterator<Item = (usize, char)>,
	{
		iter.next().unwrap_or((key.len(), ' ')).0
	}
}

fn intra_word_removable(c: char) -> bool {
	match c {
		'々' | '_' | '\'' => true,
		'・' | '᐀' => false, // For our purposes `・` and `᐀` are word separators
		_ if is_word_mark(c) => true,
		_ => false,
	}
}

fn is_searchable(c: char) -> bool {
	is_kanji(c) || is_hiragana(c)
}

fn is_word_split(c: char) -> bool {
	match c {
		'・' | '᐀' | '~' | '～' => true,
		_ if is_japanese_punctuation(c) => true,
		_ => !char::is_alphanumeric(c),
	}
}

/// Implement searching for the main database.
impl Search for Root {
	fn search_terms<S>(&self, log: &Logger, query: S, options: &SearchOptions) -> (Vec<(usize, TermRow)>, usize)
	where
		S: AsRef<str>,
	{
		use std::ops::Sub;

		// Helper to enumerate all keywords for a term by index.
		fn keys_for(db: &Root, index: usize) -> impl Iterator<Item = &String> {
			let entry = &db.terms[index];
			vec![&entry.expression, &entry.reading].into_iter().chain(
				entry
					.forms
					.iter()
					.map(|x| vec![&x.expression, &x.reading].into_iter())
					.flatten(),
			)
		}

		// TODO: prioritize search terms based on the query mode
		// TODO: implement fuzzy searching

		// TODO: use split on the query to allow multiple words
		let query = crate::kana::normalize_search_string(query, true);

		// No matter what, never go past this many results.
		const HARD_LIMIT: usize = 50000;

		// If no limit is specified, use this.
		let limit = if options.limit > 0 { options.limit } else { 100 };

		let max_count = std::cmp::min(options.offset + limit, HARD_LIMIT);

		let log = log.new(o!("search" => query.clone()));
		time!(t_query);
		trace!(log, "searching with max count {}...", max_count);

		// Exact matches
		let exact_matches = self.index.search_term_word_by_prefix(&query, true);
		let count = exact_matches.len();

		trace!(log, "... found {} exact matches", exact_matches.len(); t_query);

		// Exact "prefix" matches
		let (prefix_matches, count) = if options.mode.includes_prefix() && count < max_count {
			let set = self.index.search_term_word_by_prefix(&query, false);
			let set = set.sub(&exact_matches);
			let count = count + set.len();
			(set, count)
		} else {
			(Default::default(), count)
		};

		trace!(log, "... found {} prefix matches", prefix_matches.len(); t_query);

		// Exact "suffix" matches
		let (suffix_matches, count) = if options.mode.includes_suffix() && count < max_count {
			let check_index = |index: &usize| -> bool {
				if exact_matches.contains(index) || prefix_matches.contains(index) {
					false
				} else {
					keys_for(self, *index).any(|s| s.ends_with(&query))
				}
			};
			let max_count = max_count - count;
			let candidates = self.index.search_candidates_by_suffix(&query).into_iter().sorted();
			let matches = candidates.filter(check_index).take(max_count);
			let set = matches.collect::<HashSet<usize>>();
			let count = count + set.len();
			(set, count)
		} else {
			(Default::default(), count)
		};

		trace!(log, "... found {} suffix matches", suffix_matches.len(); t_query);

		// Do a keyword search for "contains" and fuzzy search.
		let is_contains = options.mode == SearchMode::Contains;
		let indexes_by_keyword = if (is_contains || options.fuzzy) && count < max_count {
			self.index.indexes_by_keyword(&query)
		} else {
			Default::default()
		};

		if indexes_by_keyword.len() > 0 {
			trace!(log, "... matched {} by keyword", indexes_by_keyword.len(); t_query);
		}

		// Exact "contains" matches
		let (contain_matches, count) = if is_contains && count < max_count {
			let check_index = |index: &usize| -> bool {
				if exact_matches.contains(index) || prefix_matches.contains(index) || suffix_matches.contains(index) {
					false
				} else {
					keys_for(self, *index).any(|s| s.contains(&query))
				}
			};
			let max_count = max_count - count;
			let candidates = indexes_by_keyword.iter().cloned().sorted();
			let matches = candidates.filter(check_index).take(max_count);
			let set = matches.collect::<HashSet<usize>>();
			let count = count + set.len();
			(set, count)
		} else {
			(Default::default(), count)
		};

		trace!(log, "... found {} contain matches", contain_matches.len(); t_query);

		let indexes = exact_matches
			.into_iter()
			.sorted()
			.chain(prefix_matches.into_iter().sorted())
			.chain(suffix_matches.into_iter().sorted())
			.chain(contain_matches.into_iter().sorted())
			.skip(options.offset)
			.take(limit);

		let mut out = Vec::new();
		for index in indexes {
			out.push((index, self.terms[index].clone()));
		}

		trace!(log, "found {} matches", out.len(); t_query);

		(out, count)
	}

	fn search_kanji<T>(&self, _log: &Logger, query: T) -> Vec<KanjiRow>
	where
		T: IntoIterator<Item = char>,
	{
		let mut out = Vec::new();
		for it in query.into_iter() {
			if let Some(index) = self.index.search_kanji(it) {
				out.push(self.kanji[index].clone());
			}
		}
		out
	}

	fn match_prefix<S>(&self, query: S) -> Option<WordSearch>
	where
		S: AsRef<str>,
	{
		let query = query.as_ref();
		if query.len() == 0 {
			None
		} else if let Some(result) = self.search_word(query, true) {
			Some(result)
		} else {
			let chars = query.char_indices().map(|(i, _)| i).collect::<Vec<_>>();
			let mut chars = &chars[1..];
			while let Some(&index) = chars.last() {
				let query = &query[0..index];
				if let Some(result) = self.search_word(query, true) {
					return Some(result);
				}
				chars = &chars[0..chars.len() - 1];
			}
			None
		}
	}

	fn search_word<S>(&self, word: S, deinflect: bool) -> Option<WordSearch>
	where
		S: AsRef<str>,
	{
		let original = word.as_ref().to_string();
		let word = to_hiragana(&original);
		if word.len() == 0 {
			None
		} else if deinflect && japanese::can_deinflect(&word) {
			for form in japanese::deinflect(&word) {
				let set = self.index.search_term_word_by_prefix(&form.term, true);
				if set.len() > 0 {
					let indexes = set.into_iter().sorted();
					return Some(WordSearch {
						text: original,
						term: form.term,
						list: indexes.map(|x| (x, self.terms[x].clone())).collect(),
						info: form.from,
					});
				}
			}
			None
		} else {
			let set = self.index.search_term_word_by_prefix(&word, true);
			if set.len() > 0 {
				let indexes = set.into_iter().sorted();
				Some(WordSearch {
					text: original,
					term: word,
					list: indexes.map(|x| (x, self.terms[x].clone())).collect(),
					info: vec![],
				})
			} else {
				None
			}
		}
	}
}

// spell-checker: disable

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_search_strings() {
		// Non-searchable strings
		assert_eq!(search_strings("").len(), 0);
		assert_eq!(search_strings("  ").len(), 0);
		assert_eq!(search_strings("123 456").len(), 0);

		// Simple words
		assert_eq!(search_strings("tomodachi"), vec!["ともだち"]);
		assert_eq!(search_strings("ともだち"), vec!["ともだち"]);
		assert_eq!(search_strings("トモダチ"), vec!["ともだち"]);
		assert_eq!(search_strings("友達"), vec!["友達"]);
		assert_eq!(
			search_strings("ともだち友達トモダチdesu"),
			vec!["ともだち友達ともだちです"]
		);

		// Intra-word separators
		assert_eq!(search_strings("to_mo-da''123''chi"), vec!["ともだち"]);
		assert_eq!(search_strings("とーもヽヾだゝゞち"), vec!["ともだち"]);
		assert_eq!(search_strings("トモダチ"), vec!["ともだち"]);
		assert_eq!(search_strings("友x123x々達"), vec!["友達"]);

		// Word separators
		assert_eq!(
			search_strings("to mo,da/chi~ta・chi"),
			vec!["と", "も", "だ", "ち", "た", "ち"]
		);
		assert_eq!(
			search_strings("と・も᐀だ～ち　た『ち』"),
			vec!["と", "も", "だ", "ち", "た", "ち"]
		);
		assert_eq!(
			search_strings("と も,だ’ち~た(ち)"),
			vec!["と", "も", "だ", "ち", "た", "ち"]
		);
	}

	#[test]
	fn test_search_keys() {
		fn check<T: IntoIterator<Item = S>, S: Into<String>>(key: &str, expected: T) {
			use itertools::*;
			let mut vec = search_keys(key)
				.map(|SearchKey(c0, c1)| {
					if c1 != '\0' {
						vec![c0, c1].into_iter().join("")
					} else {
						c0.to_string()
					}
				})
				.collect::<Vec<_>>();
			let mut expected = expected.into_iter().map(|x| x.into()).collect::<Vec<_>>();
			vec.sort();
			expected.sort();
			assert_eq!(vec, expected);
		}

		assert_eq!(search_keys("").count(), 0);
		check("とも", vec!["と", "も", "とも"]);
		check(
			"ともだち",
			vec![
				"と", "も", "だ", "ち", "とも", "とだ", "とち", "もだ", "もち", "だち",
			],
		);
		check("友達とも", vec!["友", "達", "と", "も", "とも"]);
	}
}

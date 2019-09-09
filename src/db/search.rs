use std::collections::HashSet;

use super::tables::*;
use crate::kana::*;

use itertools::*;

/// Provides a first-pass broad key to index words for a `contains` type
/// search.
///
/// A key `(c0, c2)` will index all words that contain `c0` and `c2` anywhere in
/// the word, provided `c2` appears anywhere after `c1`.
///
/// For a single char index, `c2` will be `\0`.
#[derive(Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct SearchKey(char, char);

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

/// Normalize the input, split it and filter out unsearchable characters.
///
/// Normalization occurs as following:
/// - The text is normalized to the Unicode NFC form and converted to lowercase.
/// - Katakana and Romaji are converted to Hiragana.
/// - Intraword punctuation chars and `ー` are removed.
/// - The Katakana `ー` is also removed.
/// - The result is split by punctuation and spaces.
/// - Non-kanji and non-kana characters are removed.
#[allow(dead_code)]
fn search_strings<S: AsRef<str>>(query: S) -> Vec<String> {
	let text = normalize_search_string(query, true);
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

/// Performs the basic normalization for a search string.
///
/// This performs Unicode normalization (to NFC) and lowercases the input.
///
/// If `japanese` is true, will also convert the input to hiragana.
pub fn normalize_search_string<S: AsRef<str>>(query: S, japanese: bool) -> String {
	use unicode_normalization::UnicodeNormalization;

	// First step, normalize the string. We use NFC to make sure accented
	// characters are a single codepoint.
	let text = query.as_ref().trim().to_lowercase().nfc().collect::<String>();
	let text = if japanese { to_hiragana(text) } else { text };
	text
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
	fn search_terms<S1, S2>(&self, query: S1, romaji: S2, mode: SearchMode, fuzzy: bool) -> Vec<TermRow>
	where
		S1: AsRef<str>,
		S2: AsRef<str>;

	fn search_kanji<T>(&self, query: T) -> Vec<KanjiRow>
	where
		T: IntoIterator<Item = char>;
}

/// Implement searching for the main database.
impl Search for Root {
	fn search_terms<S1, S2>(&self, query: S1, _romaji: S2, mode: SearchMode, _fuzzy: bool) -> Vec<TermRow>
	where
		S1: AsRef<str>,
		S2: AsRef<str>,
	{
		let mut indexes: HashSet<usize> = HashSet::new();
		let query = normalize_search_string(query, true);
		let possible_indexes = self.index.search_term_word_by_prefix(&query);
		for index in possible_indexes.into_iter() {
			let entry = &self.terms[index];
			let keys = vec![&entry.expression, &entry.reading].into_iter().chain(
				entry
					.forms
					.iter()
					.map(|x| vec![&x.expression, &x.reading].into_iter())
					.flatten(),
			);
			let mut is_match = false;
			for key in keys {
				is_match = match mode {
					SearchMode::Is => key == &query,
					SearchMode::Contains => key.contains(&query),
					SearchMode::Prefix => key.starts_with(&query),
					SearchMode::Suffix => key.ends_with(&query),
				};
				if is_match {
					break;
				}
			}

			if is_match {
				indexes.insert(index);
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
			if let Some(index) = self.index.search_kanji(it) {
				out.push(self.kanji[index].clone());
			}
		}
		out
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

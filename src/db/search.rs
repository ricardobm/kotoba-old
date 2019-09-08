use std::borrow::Cow;
use std::collections::HashSet;

use super::tables::*;
use crate::kana::*;

use itertools::*;

/// Wrapper trait for a generic input string.
pub trait InputString<'a>: Into<Cow<'a, str>> + std::fmt::Display {}

impl<'a, T> InputString<'a> for T where T: Into<Cow<'a, str>> + std::fmt::Display {}

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
fn search_strings<'a, S>(query: S) -> Vec<String>
where
	S: InputString<'a>,
{
	let text = normalize_search_string(query, true);
	let groups = text
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
pub fn normalize_search_string<'a, S>(query: S, japanese: bool) -> String
where
	S: InputString<'a>,
{
	use unicode_normalization::UnicodeNormalization;

	// First step, normalize the string. We use NFC to make sure accented
	// characters are a single codepoint.
	let text = query.into().trim().to_lowercase().nfc().collect::<String>();
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
}

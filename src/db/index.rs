use std::collections::HashMap;
use std::collections::HashSet;
use std::iter;
use std::iter::FromIterator;

use super::search::{normalize_search_string, InputString};
use crate::kana::is_kanji;

/// Serializable database index structure.
#[derive(Serialize, Deserialize)]
pub struct Index {
	/// Kanji indexes mapped by their character.
	kanji_by_char: HashMap<char, u32>,

	/// De-duplicated sorted list of japanese search words and their respective
	/// term indexes.
	///
	/// The words in this list are normalized with [normalize_search_string].
	///
	/// The purpose of this index is to allow for a binary search on a raw
	/// search term prefix.
	word_index: Vec<(String, HashSet<u32>)>,
}

impl Default for Index {
	fn default() -> Index {
		Index {
			kanji_by_char: HashMap::new(),
			word_index:    Vec::new(),
		}
	}
}

impl Index {
	/// Clear the entire index.
	pub fn clear(&mut self) {
		self.kanji_by_char.clear();
		self.word_index.clear()
	}

	// Dump index information to stdout.
	pub fn dump_info(&self) {
		println!(
			"Indexed {} kanji and {} words",
			self.kanji_by_char.len(),
			self.word_index.len()
		);
	}

	/// Search for a mapped kanji index by its char.
	pub fn search_kanji(&self, c: char) -> Option<usize> {
		if let Some(&index) = self.kanji_by_char.get(&c) {
			Some(index as usize)
		} else {
			None
		}
	}

	/// Search for mapped term index by the japanese keyword. This search
	/// will match by the prefix.
	///
	/// If the search keyword is small and could possibly return too many
	/// results, this might revert to an exact match search.
	pub fn search_term_word_by_prefix<'a, S: InputString<'a>>(&self, word: S) -> HashSet<usize> {
		let out = self.search_term_word_by_prefix_opts(word, None);
		out.into_iter().map(|x| x as usize).collect()
	}

	/// Map a kanji index by its char.
	pub fn map_kanji(&mut self, c: char, index: usize) {
		self.kanji_by_char.insert(c, index as u32);
	}

	/// Map a term index by a keyword.
	pub fn map_term_keywords<'a, T, H, S>(&mut self, mappings: T)
	where
		T: IntoIterator<Item = (usize, H)>,
		H: IntoIterator<Item = S>,
		S: InputString<'a>,
	{
		let mut table = HashMap::<String, HashSet<u32>>::new();
		for (index, words) in mappings.into_iter() {
			let index = index as u32;
			for word in words.into_iter() {
				let word = normalize_search_string(word, true);
				table
					.entry(word)
					.and_modify(|s| {
						s.insert(index);
					})
					.or_insert_with(|| HashSet::from_iter(iter::once(index)));
			}
		}
		self.word_index = table.into_iter().collect();
		self.word_index.sort_by(|x, y| x.0.cmp(&y.0));
	}

	fn search_term_word_by_prefix_opts<'a, S: InputString<'a>>(
		&self,
		word: S,
		single_mode: Option<bool>,
	) -> HashSet<u32> {
		use std::cmp::Ordering;

		// If there are this many results or more, try to perform a more
		// focused search.
		const TOO_MANY_CUTOFF: usize = 100;

		let mut indexes = HashSet::new();
		let word = normalize_search_string(word, true);
		if word.len() > 0 {
			let is_single_char = word.chars().take(2).count() == 1;
			let is_single_kanji = is_single_char && is_kanji(word.chars().next().unwrap());

			// Single mode reverts the search to a exact match instead of prefix
			// match. This can be either forced by the `single_mode` parameter
			// (in case of recursive calls, see below) or by the heuristic:
			//
			// - The search string consists of a single character.
			// - The character is not kanji (a given kanji is most likely a lot
			//   less common in the prefix of words than a hiragana).
			//
			// In any case, we may try a different decision if the results of
			// the single/not single mode end up being bad (e.g. no results or
			// too many results, as per `TOO_MANY_CUTOFF`).
			let is_single = single_mode.unwrap_or(is_single_char && !is_single_kanji);

			let cmp: Box<dyn (FnMut(&(String, HashSet<u32>)) -> Ordering)> = if is_single {
				Box::from(|it: &(String, HashSet<u32>)| it.0.cmp(&word))
			} else {
				Box::from(|it: &(String, HashSet<u32>)| {
					if it.0.starts_with(&word) {
						std::cmp::Ordering::Equal
					} else {
						it.0.cmp(&word)
					}
				})
			};

			if let Ok(pos) = self.word_index.binary_search_by(cmp) {
				let last = self.word_index.len() - 1;
				let mut sta = pos;
				let mut end = pos;
				while !is_single && sta > 0 && self.word_index[sta - 1].0.starts_with(&word) {
					sta -= 1;
				}
				while !is_single && end < last && self.word_index[end + 1].0.starts_with(&word) {
					end += 1;
				}

				for i in sta..=end {
					indexes = indexes.union(&self.word_index[i].1).cloned().collect();
				}
			}

			// We want to repeat the search using a different decision for
			// the `is_single` behavior above in case we god bad results, but
			// only if `single_mode` is None, meaning we are not a recursive
			// call.
			//
			// A bad result is either zero or too many, as per `TOO_MANY_CUTOFF`.
			//
			//     +=========++================= Results =================+
			//     | Single? ||   too many results  |  too few results    |
			//     +---------++---------------------+---------------------+
			//     |  true   ||         ---         |      repeat(F)      |
			//     +---------++---------------------+---------------------+
			//     |  false  ||      repeat(T)      |         ---         |
			//     +---------++---------------------+---------------------+
			//
			if single_mode.is_none() {
				if indexes.len() == 0 && is_single {
					// We got zero results, try to broaden the search and use
					// whatever is returned.
					self.search_term_word_by_prefix_opts(word, Some(false))
				} else if indexes.len() > TOO_MANY_CUTOFF && !is_single {
					// We got too many results, try to tighten the search...
					let fewer = self.search_term_word_by_prefix_opts(word, Some(true));
					if fewer.len() > 0 {
						// ...we got a match, use it!
						fewer
					} else {
						// ...the exact match got nothing, this result may
						// be useless but at least is better than nothing.
						indexes
					}
				} else {
					indexes // our result is either good enough or all we have
				}
			} else {
				indexes // we are a recursive call, just return what we have
			}
		} else {
			Default::default() // empty search term returns empty
		}
	}
}

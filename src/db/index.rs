use std::collections::HashMap;
use std::collections::HashSet;
use std::iter;
use std::iter::FromIterator;

use super::search::normalize_search_string;

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
	/// Assumes the search keyword is already normalized.
	///
	/// If the search keyword is small and could possibly return too many
	/// results, this might revert to an exact match search.
	pub fn search_term_word_by_prefix<S: AsRef<str>>(&self, word: S) -> HashSet<usize> {
		// If there are this many results or more, try to perform a more
		// focused search.
		const TOO_MANY_CUTOFF: usize = 10000;

		let word = word.as_ref();

		// First try to do a broader search by prefix...
		let out = self.search_term_word_by_prefix_opts(word, false);

		// ...if there are too many results
		let out = if out.len() > TOO_MANY_CUTOFF {
			// then try to narrow the search by matching exactly.
			let redo = self.search_term_word_by_prefix_opts(word, true);
			if redo.len() > 0 {
				redo
			} else {
				out // exact match found nothing, the broader search is better than nothing
			}
		} else {
			out
		};
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
		S: AsRef<str>,
	{
		let mut table = HashMap::<String, HashSet<u32>>::new();
		for (index, words) in mappings.into_iter() {
			let index = index as u32;
			for word in words.into_iter() {
				let word = normalize_search_string(word.as_ref(), true);
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

	fn search_term_word_by_prefix_opts<S: AsRef<str>>(&self, word: S, single_mode: bool) -> HashSet<u32> {
		use std::cmp::Ordering;

		let mut indexes = HashSet::new();
		let word = word.as_ref();
		if word.len() > 0 {
			let cmp: Box<dyn (FnMut(&(String, HashSet<u32>)) -> Ordering)> = if single_mode {
				// For `single_mode` use a straightforward comparison
				Box::from(|it: &(String, HashSet<u32>)| it.0.as_str().cmp(word))
			} else {
				// In prefix mode, first compare the prefix
				Box::from(|it: &(String, HashSet<u32>)| {
					if it.0.starts_with(word) {
						std::cmp::Ordering::Equal
					} else {
						it.0.as_str().cmp(word)
					}
				})
			};

			if let Ok(pos) = self.word_index.binary_search_by(cmp) {
				let last = self.word_index.len() - 1;
				let mut sta = pos;
				let mut end = pos;

				// In prefix mode, expand the result range to include all
				// prefixed results
				while !single_mode && sta > 0 && self.word_index[sta - 1].0.starts_with(word) {
					sta -= 1;
				}
				while !single_mode && end < last && self.word_index[end + 1].0.starts_with(word) {
					end += 1;
				}

				for i in sta..=end {
					for &key in self.word_index[i].1.iter() {
						indexes.insert(key);
					}
				}
			}

			indexes
		} else {
			Default::default() // empty search term returns empty
		}
	}
}

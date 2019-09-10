use std::collections::HashSet;

use super::search::normalize_search_string;
use super::search::{search_keys, SearchKey};

use fnv::{FnvHashMap, FnvHashSet};

/// Serializable database index structure.
#[derive(Serialize, Deserialize)]
pub struct Index {
	/// Kanji indexes mapped by their character.
	kanji_by_char: FnvHashMap<char, u32>,

	/// De-duplicated sorted list of japanese search words and their respective
	/// term indexes.
	///
	/// The words in this list are normalized with [normalize_search_string].
	///
	/// The purpose of this index is to allow for a binary search on a raw
	/// search term prefix.
	word_index: Vec<(String, FnvHashSet<u32>)>,

	/// Set of indexes by the last char. Used for suffix searching.
	suffix_index: FnvHashMap<String, FnvHashSet<u32>>,

	/// Set of [SearchKey] and their respective indexes generated from the
	/// dictionary words.
	key_index: FnvHashMap<SearchKey, FnvHashSet<u32>>,
}

impl Default for Index {
	fn default() -> Index {
		Index {
			kanji_by_char: Default::default(),
			word_index:    Default::default(),
			suffix_index:  Default::default(),
			key_index:     Default::default(),
		}
	}
}

impl Index {
	/// Clear the entire index.
	pub fn clear(&mut self) {
		self.kanji_by_char.clear();
		self.word_index.clear();
		self.suffix_index.clear();
		self.key_index.clear();
	}

	/// Returns if the index is empty.
	pub fn empty(&self) -> bool {
		self.key_index.len() == 0
	}

	// Dump index information to stdout.
	pub fn dump_info(&self) {
		use itertools::*;

		println!(
			"\nIndexed {} kanji and {} words (keys: {}, suffixes: {})",
			self.kanji_by_char.len(),
			self.word_index.len(),
			self.key_index.len(),
			self.suffix_index.len(),
		);

		// Suffix index
		let total = self.suffix_index.values().fold(0, |acc, x| acc + x.len());
		let avg = total / self.suffix_index.len();
		let max = self.suffix_index.values().map(|x| x.len()).max().unwrap();
		let min = self.suffix_index.values().map(|x| x.len()).min().unwrap();
		println!(
			"\nSuffix index has {} total entries / {} avg / {} max / {} min",
			total, avg, max, min,
		);

		println!(
			"-> {}",
			self.suffix_index
				.iter()
				.sorted_by_key(|x| -(x.1.len() as i64))
				.take(5)
				.map(|x| format!("{}: {}", x.0, x.1.len()))
				.join(", ")
		);

		let total = self.key_index.values().fold(0, |acc, x| acc + x.len());
		let avg = total / self.key_index.len();
		let max = self.key_index.values().map(|x| x.len()).max().unwrap();
		let min = self.key_index.values().map(|x| x.len()).min().unwrap();
		println!(
			"\nKey index has {} total entries / {} avg / {} max / {} min",
			total, avg, max, min,
		);

		println!(
			"{}  |\n",
			self.key_index
				.iter()
				.sorted_by_key(|x| -(x.1.len() as i64))
				.take(50)
				.enumerate()
				.map(|x| format!(
					"{}{}\t{:6}",
					if x.0 % 10 == 0 {
						if x.0 > 0 {
							"  |\n  |  "
						} else {
							"\n  |  "
						}
					} else {
						"  |  "
					},
					(x.1).0,
					(x.1).1.len()
				))
				.join("")
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
	/// will match by the prefix or by full match.
	///
	/// Assumes the search keyword is already normalized.
	pub fn search_term_word_by_prefix<S: AsRef<str>>(&self, word: S, full_match: bool) -> HashSet<usize> {
		let word = word.as_ref();
		let out = self.do_search_term_word_by_prefix(word, full_match);
		out.into_iter().map(|x| x as usize).collect()
	}

	/// Search for candidate indexes using the word suffix.
	///
	/// Assumes the search keyword is already normalized.
	///
	/// This searches for one or two character suffix matches, so it does not
	/// guarantee a complete match.
	pub fn search_candidates_by_suffix<S: AsRef<str>>(&self, word: S) -> HashSet<usize> {
		let word = word.as_ref();
		let key = word.chars().rev().take(2).collect::<String>();
		if let Some(indexes) = self.suffix_index.get(&key) {
			indexes.iter().map(|x| *x as usize).collect()
		} else {
			Default::default()
		}
	}

	/// Search for candidate indexes given the keyword. Assumes the keyword has
	/// already been normalized.
	///
	/// This uses [search_keys] to generate all candidate keys for the given
	/// keyword and generates a set with the intersection of all keys.
	pub fn indexes_by_keyword<S: AsRef<str>>(&self, word: S) -> HashSet<usize> {
		// Stop if we get a result set this small.
		const SMALL_ENOUGH: usize = 100;

		let mut out = FnvHashSet::default();
		let mut first = true;
		for key in search_keys(word) {
			if let Some(indexes) = self.key_index.get(&key) {
				if first {
					out = indexes.clone();
					first = false;
				} else {
					out = out.intersection(indexes).cloned().collect();
				}
				if out.len() < SMALL_ENOUGH {
					break;
				}
			}
		}

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
		// Temporary hashmap to build the sorted keyword array used for the
		// prefix search.
		let mut table = FnvHashMap::<String, FnvHashSet<u32>>::default();

		for (index, words) in mappings.into_iter() {
			let index = index as u32;

			if index > 0 && index % 100000 == 0 {
				println!("...{}", index);
			}

			for word in words.into_iter() {
				let word = normalize_search_string(word.as_ref(), true);

				// Map word for the "contains" search
				for key in search_keys(&word) {
					self.key_index
						.entry(key)
						.and_modify(|s| {
							s.insert(index);
						})
						.or_insert_with(|| {
							let mut set = FnvHashSet::default();
							set.insert(index);
							set
						});
				}

				// Map word for the suffix search
				let suffix1 = word.chars().rev().take(1).collect::<String>();
				let suffix2 = word.chars().rev().take(2).collect::<String>();
				for it in vec![suffix1, suffix2].into_iter() {
					self.suffix_index
						.entry(it.clone())
						.and_modify(|s| {
							s.insert(index);
						})
						.or_insert_with(|| {
							let mut set = FnvHashSet::default();
							set.insert(index);
							set
						});
				}

				// Map word for the prefix search
				table
					.entry(word)
					.and_modify(|s| {
						s.insert(index);
					})
					.or_insert_with(|| {
						let mut set = FnvHashSet::default();
						set.insert(index);
						set
					});
			}
		}
		self.word_index = table.into_iter().collect();
		self.word_index.sort_by(|x, y| x.0.cmp(&y.0));
	}

	fn do_search_term_word_by_prefix<S: AsRef<str>>(&self, word: S, full_match: bool) -> HashSet<u32> {
		use std::cmp::Ordering;

		let mut indexes = HashSet::new();
		let word = word.as_ref();
		if word.len() > 0 {
			let cmp: Box<dyn (FnMut(&(String, FnvHashSet<u32>)) -> Ordering)> = if full_match {
				// For `full_match` use a straightforward comparison
				Box::from(|it: &(String, FnvHashSet<u32>)| it.0.as_str().cmp(word))
			} else {
				// In prefix mode, first compare the prefix
				Box::from(|it: &(String, FnvHashSet<u32>)| {
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
				while !full_match && sta > 0 && self.word_index[sta - 1].0.starts_with(word) {
					sta -= 1;
				}
				while !full_match && end < last && self.word_index[end + 1].0.starts_with(word) {
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

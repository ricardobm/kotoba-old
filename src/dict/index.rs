pub struct IndexTable {
	entries: Vec<(String, usize)>,
}

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub enum SearchMode {
	Exact,
	Contains,
	Prefix,
	Suffix,
}

impl IndexTable {
	pub fn new() -> IndexTable {
		IndexTable { entries: Vec::new() }
	}

	pub fn clear(&mut self) {
		self.entries.clear();
	}

	pub fn insert<S: AsRef<str>>(&mut self, text: S, pos: usize) {
		self.entries.push((text.as_ref().to_lowercase(), pos));
	}

	pub fn search<'a, S: AsRef<str>>(&'a self, text: S, mode: SearchMode) -> IndexIterator<'a> {
		IndexIterator {
			table: self,
			index: 0,
			query: text.as_ref().to_lowercase(),
			mode:  mode,
		}
	}
}

pub struct IndexIterator<'a> {
	table: &'a IndexTable,
	index: usize,
	query: String,
	mode:  SearchMode,
}

impl<'a> Iterator for IndexIterator<'a> {
	type Item = (&'a str, usize);

	fn next(&mut self) -> Option<(&'a str, usize)> {
		let len = self.table.entries.len();
		while self.index < len {
			let (text, index) = &self.table.entries[self.index];
			self.index = self.index + 1;
			let is_match = match self.mode {
				SearchMode::Exact => text == &self.query,
				SearchMode::Contains => text.contains(&self.query),
				SearchMode::Prefix => text.starts_with(&self.query),
				SearchMode::Suffix => text.ends_with(&self.query),
			};
			if is_match {
				return Some((text.as_str(), *index));
			}
		}
		None
	}
}

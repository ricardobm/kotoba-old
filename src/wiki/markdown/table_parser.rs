use std::fmt;

use super::Span;

pub enum Row<'a> {
	Delimiter(usize),
	Content(TableRow<'a>),
}

#[derive(Clone)]
pub enum TableAlign {
	Normal,
	Left,
	Right,
	Center,
}

impl TableAlign {
	pub fn fmt_attr(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			TableAlign::Normal => Ok(()),
			TableAlign::Left => write!(f, " align='left'"),
			TableAlign::Right => write!(f, " align='right'"),
			TableAlign::Center => write!(f, " align='center'"),
		}
	}
}

#[derive(Clone)]
pub struct TableRow<'a> {
	length: usize,
	source: &'a str,
}

pub fn parse_table_row<'a>(line: &'a str, check_delimiter: bool) -> Option<Row<'a>> {
	let mut source = line.trim();

	let sta_delim = if source.starts_with("|") {
		source = &source[1..];
		true
	} else {
		false
	};

	let end_delim = if source.ends_with("|") && !source.ends_with("\\|") {
		source = &source[..source.len() - 1];
		true
	} else {
		false
	};

	if source.len() == 0 {
		return None;
	}

	let buffer = Span {
		buffer:     line,
		column:     0,
		offset_sta: 0,
		offset_end: line.len(),
		indent:     0,
		quotes:     0,
	};

	let mut is_delim = check_delimiter;
	let iter = RowIterator {
		buffer: buffer.clone(),
		source: line,
		offset: 0,
	};
	let mut count = 0;
	for (text, _) in iter {
		count += 1;
		if is_delim {
			is_delim = text.text().trim().chars().all(|c| c == '-');
		}
	}

	if count > 1 || (sta_delim && end_delim) {
		// We have a table
		let row = if is_delim {
			Row::Delimiter(count)
		} else {
			Row::Content(TableRow {
				length: count,
				source: source,
			})
		};
		Some(row)
	} else {
		None
	}
}

impl<'a> TableRow<'a> {
	pub fn iter(&self, span: Span<'a>) -> RowIterator<'a> {
		RowIterator {
			buffer: span,
			source: self.source,
			offset: 0,
		}
	}

	pub fn len(&self) -> usize {
		self.length
	}
}

#[derive(Clone)]
pub struct RowIterator<'a> {
	buffer: Span<'a>,
	source: &'a str,
	offset: usize,
}

impl<'a> Iterator for RowIterator<'a> {
	type Item = (Span<'a>, TableAlign);

	fn next(&mut self) -> Option<Self::Item> {
		if self.offset >= self.source.len() {
			None
		} else {
			let mut cursor = self.offset;
			let cell = loop {
				let text = &self.source[cursor..];
				if let Some(index) = text.find("\\|") {
					cursor += index + 2;
				} else if let Some(index) = text.find('|') {
					let cell = &self.source[self.offset..index];
					self.offset = cursor + index + 1;
					break cell;
				} else {
					let cell = &self.source[self.offset..];
					self.offset = self.source.len();
					break cell;
				}
			};

			let mut align = TableAlign::Normal;
			let mut cell = cell.trim();
			if cell != ":" {
				if cell.starts_with(":") {
					align = TableAlign::Left;
					cell = cell[1..].trim_start();
				}

				if cell.ends_with(":") && !cell.ends_with("\\:") {
					align = match align {
						TableAlign::Normal => TableAlign::Right,
						TableAlign::Left => TableAlign::Center,
						_ => unreachable!(),
					};
					cell = cell[..cell.len() - 1].trim_end();
				}
			}

			let cell = self.buffer.sub_range(self.buffer.text_range(cell).unwrap());
			Some((cell, align))
		}
	}
}

use std::fmt;

use super::Span;

pub enum Row<'a> {
	Delimiter(Vec<TableAlign>),
	Content(TableRow<'a>),
}

#[derive(Copy, Clone, Debug)]
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
			TableAlign::Left => write!(f, " align=\"left\""),
			TableAlign::Right => write!(f, " align=\"right\""),
			TableAlign::Center => write!(f, " align=\"center\""),
		}
	}
}

#[derive(Clone)]
pub struct TableRow<'a> {
	length: usize,
	source: Span<'a>,
}

pub fn parse_table_row<'a>(line: Span<'a>, check_delimiter: bool) -> Option<Row<'a>> {
	let mut source = line.trimmed();

	let sta_delim = if source.text().starts_with("|") {
		source = source.sub(1..);
		true
	} else {
		false
	};

	let end_delim = if source.text().ends_with("|") && !source.text().ends_with("\\|") {
		source = source.sub(..source.len() - 1);
		true
	} else {
		false
	};

	if source.len() == 0 {
		return None;
	}

	let mut is_delim = check_delimiter;
	let mut count = 0;
	let iter = RowIterator {
		cursor: source.clone(),
		align:  check_delimiter,
	};
	for (text, _) in iter {
		count += 1;
		if is_delim {
			is_delim = text.text().trim().chars().all(|c| c == '-');
		}
	}

	if count > 1 || (sta_delim && end_delim) {
		// We have a table
		let row = if is_delim {
			let iter = RowIterator {
				cursor: source.clone(),
				align:  check_delimiter,
			};
			Row::Delimiter(iter.map(|c| c.1).collect())
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
	pub fn from_line(line: Span<'a>) -> TableRow<'a> {
		TableRow {
			source: line,
			length: 1,
		}
	}

	pub fn iter(&self) -> RowIterator<'a> {
		RowIterator {
			cursor: self.source.clone(),
			align:  false,
		}
	}

	pub fn len(&self) -> usize {
		self.length
	}
}

#[derive(Clone)]
pub struct RowIterator<'a> {
	cursor: Span<'a>,
	align:  bool,
}

impl<'a> RowIterator<'a> {
	pub fn line_range(&self) -> (usize, usize) {
		(self.cursor.start.line, self.cursor.end.line)
	}
}

impl<'a> Iterator for RowIterator<'a> {
	type Item = (Span<'a>, TableAlign);

	fn next(&mut self) -> Option<Self::Item> {
		if self.cursor.len() == 0 {
			None
		} else {
			let mut offset = 0;
			let cell = loop {
				let text = &self.cursor.text()[offset..];
				if let Some(index) = text.find("\\|") {
					offset += index + 2;
				} else if let Some(index) = text.find('|') {
					let cell = self.cursor.sub(..offset + index);
					self.cursor = self.cursor.sub(offset + index + 1..);
					break cell;
				} else {
					let cell = self.cursor.sub(..);
					self.cursor = self.cursor.end();
					break cell;
				}
			};

			let mut align = TableAlign::Normal;
			let mut cell = cell.trimmed();
			if self.align {
				let mut text = cell.text();
				if text != ":" {
					if text.starts_with(":") {
						align = TableAlign::Left;
						cell = cell.sub_from_text(text[1..].trim_start());
						text = cell.text();
					}

					if text.ends_with(":") && !text.ends_with("\\:") {
						align = match align {
							TableAlign::Normal => TableAlign::Right,
							TableAlign::Left => TableAlign::Center,
							_ => unreachable!(),
						};
						cell = cell.sub_from_text(text[..text.len() - 1].trim_end());
					}
				}
			}

			Some((cell, align))
		}
	}
}

use std::fmt;

use super::common;

/// Span of text from the Markdown source, representing an inline block of
/// text.
///
/// Note that the raw source of the span may contain block marks (e.g. `>`),
/// stripped indentation and escape sequences, and as such is not useful for
/// direct usage.
/// Block of inline markdown text that can be independently parsed into [Inline]
/// elements.
#[derive(Clone)]
pub struct Span<'a> {
	/// Full raw source text.
	pub buffer: &'a str,
	/// Column for this span's text start.
	pub column: usize,
	/// Start offset for the span's text.
	pub offset_sta: usize,
	/// End offset for the span's text.
	pub offset_end: usize,
	/// Maximum base indentation to be removed from raw text.
	pub indent: usize,
	/// Number of quotation levels (e.g. `>`) to remove from text.
	pub quotes: usize,
}

pub type Range = std::ops::Range<usize>;

impl<'a> Span<'a> {
	pub fn empty() -> Span<'static> {
		Span {
			buffer:     "",
			column:     0,
			offset_sta: 0,
			offset_end: 0,
			indent:     0,
			quotes:     0,
		}
	}

	#[inline(always)]
	pub fn text(&self) -> &'a str {
		&self.buffer[self.offset_sta..self.offset_end]
	}

	pub fn sub_range(&self, range: Range) -> Span<'a> {
		let prefix = &self.buffer[self.offset_sta..range.start];
		Span {
			buffer:     self.buffer,
			column:     self.column + common::text_column(prefix, self.column),
			offset_sta: range.start,
			offset_end: range.end,
			indent:     self.indent,
			quotes:     self.quotes,
		}
	}

	/// Convert a block of text back to an offset in the Span's buffer.
	///
	/// Returns `None` if the text does not belong to the buffer. Note that
	/// this can happen for spaces returned by [SpanIter] to pad tab-width.
	pub fn text_range(&self, text: &'a str) -> Option<Range> {
		let buf_sta = self.buffer.as_ptr() as usize;
		let buf_end = buf_sta + self.buffer.len();
		let txt_len = text.len();
		let txt_sta = text.as_ptr() as usize;
		let txt_end = txt_sta + txt_len;
		if txt_sta < buf_sta || txt_end > buf_end {
			None
		} else {
			let offset = txt_sta - buf_sta;
			Some(Range {
				start: offset,
				end:   offset + txt_len,
			})
		}
	}
}

impl<'a> fmt::Display for Span<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for s in self.iter() {
			write!(f, "{}", s)?;
		}
		Ok(())
	}
}

impl<'a> fmt::Debug for Span<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self.text())
	}
}

impl<'a> Span<'a> {
	/// Returns an iterator over the blocks of inline text in the [Span].
	///
	/// The iterator will skip ignored indentation and blockquote markers from
	/// each line in the span.
	///
	/// In this above case, the iterator will produce each complete line in
	/// the [Span], possibly preceded by a `&'static str` containing spaces
	/// necessary for tab expansion (in cases where the stripped indentation
	/// falls in the "middle" of a tab character).
	///
	/// The lines produced by the iterator contain the EOL suffix (e.g. `\n`
	/// or `\r\n`).
	///
	/// If there is nothing to skip, the iterator will produce a single item
	/// with the whole text.
	pub fn iter(&self) -> SpanIter<'a> {
		SpanIter {
			source: self.text(),
			column: 0,
			indent: self.indent,
			quotes: self.quotes,
		}
	}
}

/// Iterator for the text in a [Span].
pub struct SpanIter<'a> {
	source: &'a str,
	column: usize,
	indent: usize,
	quotes: usize,
}

impl<'a> Iterator for SpanIter<'a> {
	type Item = &'a str;

	fn next(&mut self) -> Option<Self::Item> {
		if self.source.len() == 0 {
			None
		} else if self.indent == 0 && self.quotes == 0 {
			// If we have nothing to skip, just return the whole text at once.
			let text = self.source;
			self.source = "";
			Some(text)
		} else {
			let mut source = self.source;

			// If we are at the beginning of a line, then we strip any block
			// quote marks and base indentation from the text.
			if self.column == 0 {
				let mut column = self.column;

				// Strip quote markers from the source text.
				for _ in 0..self.quotes {
					// We don't want to trim `source` unless we stripped a quote
					// marker.
					let (text, col) = common::trim_start(source, column);
					let (new_text, new_column) = if text.starts_with("> ") {
						(&text[2..], col + 2)
					} else if text.starts_with(">") {
						(&text[1..], col + 1)
					} else {
						break;
					};
					source = new_text;
					column = new_column;
				}

				// Strip the indentation level from the source text.
				let mut indent = self.indent;
				let mut indent_to_return = "";
				while indent > 0 {
					if source.starts_with("\t") {
						let tw = common::TAB_WIDTH - (column % common::TAB_WIDTH);
						source = common::skip_chars(source, 1);
						column += tw;
						if indent >= tw {
							indent -= tw;
						} else {
							// if the tab width is greater than the indentation
							// we must skip, we generate a block of spaces to
							// compensate
							indent_to_return = &("    "[0..(tw - indent)]);
							indent = 0;
						}
					} else if let Some(chr) = source.chars().next() {
						if chr.is_whitespace() {
							source = common::skip_chars(source, 1);
							column += 1;
							indent -= 1;
						} else {
							break;
						}
					} else {
						break;
					}
				}

				self.source = source;
				self.column = column;
				if indent_to_return.len() > 0 {
					return Some(indent_to_return);
				}
			}

			let source = self.source;
			let line_size = source.find(|c| c == '\n' || c == '\r');
			let line_size = match line_size {
				None => source.len(),
				Some(size) if (&source[size..]).starts_with("\r\n") => size + 2,
				Some(size) => size + 1,
			};

			self.source = &source[line_size..];
			self.column = 0;

			Some(&source[0..line_size])
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_markdown_span_iter() {
		fn check(indent: usize, quotes: usize, source: &str, expected: Vec<&str>) {
			let iter = SpanIter {
				source: source,
				column: 0,
				indent: indent,
				quotes: quotes,
			};
			let actual: Vec<_> = iter.collect();
			assert_eq!(expected, actual);
		}

		// Empty iterator:
		check(0, 0, "", vec![]);
		check(3, 4, "", vec![]);

		// Iteration without striping:
		check(0, 0, "ABC\nDEF", vec!["ABC\nDEF"]);

		// Iteration by line:
		check(
			1,
			1,
			"ABC\nDEF\r123\r\n456\n",
			vec!["ABC\n", "DEF\r", "123\r\n", "456\n"],
		);

		// Strip indent:

		check(
			4,
			0,
			"L0\n L1\n  L2\n   L3\n    L4\n     L5\n",
			vec!["L0\n", "L1\n", "L2\n", "L3\n", "L4\n", " L5\n"],
		);

		check(
			6,
			0,
			"\t  L0\n\t   L1\n\t\tL2\n \t\t L3\n  \t \t  L4\n   \t    \t L5\n",
			vec![
				"L0\n",
				" L1\n",
				"  ",
				"L2\n",
				"  ",
				" L3\n",
				"  ",
				"  L4\n",
				"  \t L5\n",
			],
		);

		// Strip quotes:
		check(
			0,
			1,
			"> L1\n>L2\n>  L3\n> > L4\n>> L5\n",
			vec!["L1\n", "L2\n", " L3\n", "> L4\n", "> L5\n"],
		)
	}
}

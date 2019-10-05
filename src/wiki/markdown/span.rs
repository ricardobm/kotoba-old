use std::fmt;

use super::common;
use super::{Pos, Range};

/// Span of text from the Markdown source, representing an inline block of
/// text.
///
/// Note that the raw source of the span may contain block marks (e.g. `>`),
/// stripped indentation and escape sequences, and as such is not useful for
/// direct usage.
/// Block of inline markdown text that can be independently parsed into [Inline]
/// elements.
#[derive(Clone, Eq)]
pub struct Span<'a> {
	/// Full raw source text.
	pub buffer: &'a str,
	/// Start position for the Span.
	pub start: Pos,
	/// End position for the Span.
	pub end: Pos,
	/// Maximum base indentation to be removed from raw text.
	pub indent: usize,
	/// Number of quotation levels (e.g. `>`) to remove from text.
	pub quotes: usize,
	/// Is this block of text inside a loose paragraph?
	pub loose: Option<bool>,
}

// NOTE: both Hash and PartialEq implementations are here just to support the
// link reference definition label comparison rules.

impl<'a> std::hash::Hash for Span<'a> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		for s in self.iter_folded() {
			s.hash(state);
		}
	}
}

impl<'a> std::cmp::PartialEq for Span<'a> {
	fn eq(&self, other: &Span<'a>) -> bool {
		let (mut s1, mut s2) = (self.iter_folded(), other.iter_folded());
		while let Some(s1) = s1.next() {
			if let Some(s2) = s2.next() {
				if s1 != s2 {
					return false;
				}
			} else {
				return false;
			}
		}
		if let Some(_) = s2.next() {
			false
		} else {
			true
		}
	}
}

impl<'a> Default for Span<'a> {
	fn default() -> Span<'static> {
		Span {
			buffer: "",
			start:  Default::default(),
			end:    Default::default(),
			indent: 0,
			quotes: 0,
			loose:  None,
		}
	}
}

impl<'a> Span<'a> {
	pub fn new(buffer: &'a str, start: Pos, end: Pos) -> Span<'a> {
		Span {
			buffer,
			start,
			end,
			..Default::default()
		}
	}

	pub fn len(&self) -> usize {
		self.end.offset - self.start.offset
	}

	pub fn iter_folded(&self) -> impl Iterator<Item = unicase::UniCase<String>> + 'a {
		use regex::Regex;
		use unicode_normalization::UnicodeNormalization;

		lazy_static! {
			static ref RE_SPACES: Regex = Regex::new(r"\s+").unwrap();
		}

		self.iter().map(|s| {
			let s: String = RE_SPACES.replace_all(s.trim(), " ").nfc().collect();
			unicase::UniCase::unicode(s)
		})
	}

	pub fn trimmed(&self) -> Span<'a> {
		if self.buffer != "" {
			let text = self.text().trim();
			if let Some(range) = self.text_range(text) {
				self.sub(range)
			} else {
				unreachable!()
			}
		} else {
			Span::default()
		}
	}

	#[inline(always)]
	pub fn text(&self) -> &'a str {
		&self.buffer[self.start.offset..self.end.offset]
	}

	pub fn sub_text<T>(&self, range: T) -> &'a str
	where
		T: std::ops::RangeBounds<Pos>,
	{
		let range = super::range_from_pos(&range, self.buffer.len());
		&self.buffer[range.start..range.end]
	}

	pub fn end(&self) -> Span<'a> {
		self.sub(self.len()..)
	}

	pub fn sub_from_text(&self, s: &str) -> Span<'a> {
		self.sub(self.text_range(s).unwrap())
	}

	pub fn sub<T: std::ops::RangeBounds<usize>>(&self, range: T) -> Span<'a> {
		let Range { start, end } = super::range_from(&range, self.len());
		Span {
			buffer: self.buffer,
			start:  self.start.advance(self.buffer, start),
			end:    self.start.advance(self.buffer, end),
			indent: self.indent,
			quotes: self.quotes,
			loose:  None,
		}
	}

	pub fn sub_pos<T: std::ops::RangeBounds<Pos>>(&self, range: T) -> Span<'a> {
		let mut range = super::range_from_pos(&range, self.buffer.len());
		debug_assert!(range.start >= self.start.offset);
		debug_assert!(range.end <= self.end.offset);
		range.start -= self.start.offset;
		range.end -= self.start.offset;
		self.sub(range)
	}

	/// Convert a block of text back to an offset in the Span's buffer.
	///
	/// Returns `None` if the text does not belong to the buffer. Note that
	/// this can happen for spaces returned by [SpanIter] to pad tab-width.
	pub fn text_range(&self, text: &'a str) -> Option<Range> {
		let buffer = self.text();
		if buffer == "" && text == "" {
			return Some(Range { start: 0, end: 0 });
		}

		let buf_sta = buffer.as_ptr() as usize;
		let buf_end = buf_sta + buffer.len();
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
			span:     self.clone(),
			cursor:   self.start,
			maxpos:   self.end,
			next_eol: None,
			pending:  "",
			stripped: false,
		}
	}
}

/// Iterator for the text in a [Span].
#[derive(Clone)]
pub struct SpanIter<'a> {
	span:   Span<'a>,
	cursor: Pos,
	maxpos: Pos,

	next_eol: Option<Pos>,
	pending:  &'static str,
	stripped: bool,
}

impl<'a> SpanIter<'a> {
	#[inline(always)]
	pub fn pos(&self) -> Pos {
		self.cursor
	}

	pub fn span(&self) -> &Span<'a> {
		&self.span
	}

	pub fn restore_from(&mut self, iter: &SpanIter<'a>) {
		self.cursor = iter.cursor;
		self.next_eol = iter.next_eol;
		self.pending = iter.pending;
		self.stripped = iter.stripped;
	}

	pub fn skip_to(&mut self, pos: Pos) {
		debug_assert!(pos >= self.cursor && pos <= self.maxpos);
		self.cursor = pos;
		self.pending = "";
		self.stripped = false;
	}

	#[inline(always)]
	pub fn at_end(&self) -> bool {
		self.pending.len() == 0 && self.cursor.offset >= self.maxpos.offset
	}

	#[inline(always)]
	pub fn chunk(&mut self) -> &'a str {
		if self.pending.len() > 0 {
			self.pending
		} else if self.span.indent == 0 && self.span.quotes == 0 {
			&self.span.buffer[self.cursor.offset..self.maxpos.offset]
		} else {
			self.skip_ignored();
			if self.pending.len() > 0 {
				self.pending
			} else {
				&self.span.buffer[self.cursor.offset..self.eol().offset]
			}
		}
	}

	pub fn skip_bytes(&mut self, mut bytes: usize) {
		while !self.at_end() && bytes > 0 && bytes >= self.chunk().len() {
			bytes -= self.chunk().len();
			self.skip_chunk();
		}
		if self.pending.len() > 0 {
			self.cursor.column = common::text_column(&self.pending[..bytes], self.cursor.column);
			self.pending = &self.pending[bytes..];
		} else {
			self.cursor.skip_len(self.span.buffer, bytes);
		}
	}

	pub fn skip_chunk(&mut self) {
		if self.pending.len() > 0 {
			self.cursor.column = common::text_column(self.pending, self.cursor.column);
			self.pending = "";
		} else if self.span.indent == 0 && self.span.quotes == 0 {
			self.cursor = self.maxpos;
		} else {
			self.cursor = self.eol();
			self.stripped = false;
		}
	}

	fn eol(&mut self) -> Pos {
		if let Some(eol) = self.next_eol {
			if eol.offset > self.cursor.offset {
				return eol;
			}
		}
		let eol = self.cursor.next_line(self.span.buffer);
		let eol = if eol.offset <= self.maxpos.offset {
			eol
		} else {
			self.maxpos
		};
		self.next_eol = Some(eol);
		eol
	}

	/// Skip ignored indentation and blockquote marks from the start of a line.
	///
	/// Does nothing if it is not at the start of a line.
	fn skip_ignored(&mut self) {
		if self.pending.len() > 0 || self.cursor.column != 0 || self.stripped {
			return;
		}

		self.stripped = true;

		// Strip quote markers from the source text.
		for _ in 0..self.span.quotes {
			let mut start = self.cursor;
			start.skip_spaces(self.span.buffer, false);
			if start.skip_if(self.span.buffer, "> ") || start.skip_if(self.span.buffer, ">") {
				if start.offset <= self.maxpos.offset {
					self.cursor = start;
				} else {
					break;
				}
			} else {
				break;
			}
		}

		// Strip the indentation level from the source text:

		let max_offset = self.maxpos.offset - self.cursor.offset;

		let mut indent = self.span.indent;
		let mut indent_to_return = "";
		let mut offset = 0;
		let mut column = self.cursor.column;
		let mut source = &self.span.buffer[self.cursor.offset..];
		while indent > 0 && offset < max_offset {
			if source.starts_with("\t") {
				let tw = common::TAB_WIDTH - (column % common::TAB_WIDTH);
				source = &source[1..];
				offset += 1;
				column += tw;
				if indent >= tw {
					indent -= tw;
				} else {
					// if the tab width is greater than the indentation
					// we must skip, we generate a block of spaces to
					// compensate
					indent_to_return = &("                "[0..(tw - indent)]);
					indent = 0;
				}
			} else if source.starts_with(" ") {
				source = &source[1..];
				offset += 1;
				column += 1;
				indent -= 1;
			} else {
				break;
			}
		}

		self.cursor.column = column;
		self.cursor.offset += offset;
		self.pending = indent_to_return;
	}

	//=========================================
	// Helper methods
	//=========================================

	pub fn next_char(&mut self) -> Option<char> {
		self.chunk().chars().next()
	}

	pub fn skip_char(&mut self) -> bool {
		let len = self.chunk().len();
		if len > 0 {
			let skip = self.chunk().char_indices().map(|x| x.0).skip(1).next().unwrap_or(len);
			self.skip_bytes(skip);
			true
		} else {
			false
		}
	}

	pub fn skip_spaces(&mut self, include_eol: bool) {
		self.cursor.skip_spaces(self.span.buffer, include_eol);
	}

	pub fn previous_char(&self) -> Option<char> {
		self.span.buffer[..self.cursor.offset].chars().last()
	}

	//=========================================
	// Search methods
	//=========================================

	// All search methods clone the iterator and iterate forward when
	// searching. As such, those methods are not limited to the current
	// chunk only.

	/// Search for text from the current position until the end of the iterator.
	pub fn search_text<T>(&self, mut search: T) -> Option<Pos>
	where
		T: FnMut(&str) -> Option<usize>,
	{
		let mut iter = self.clone();
		let mut curr = iter.pos();
		while let Some(haystack) = iter.next() {
			if let Some(index) = search(haystack) {
				let mut pos = curr;
				pos.skip(&haystack[..index]);
				return Some(pos);
			} else {
				curr = iter.pos();
			}
		}
		None
	}

	/// Search for the specific string.
	pub fn search_str(&self, needle: &str) -> Option<Pos> {
		self.search_text(|s: &str| s.find(needle))
	}

	/// Search for a char that matches the searcher.
	pub fn search_char<T>(&self, search: T) -> Option<Pos>
	where
		T: Fn(char) -> bool,
	{
		self.search_text(|s: &str| s.find(&search))
	}

	//=========================================
	// Find methods
	//=========================================

	// All methods below are limited to the current chunk only.

	pub fn find_in_chunk<T>(&mut self, search: T) -> Option<Pos>
	where
		T: Fn(&str) -> Option<usize>,
	{
		let haystack = self.chunk();
		if let Some(offset) = search(haystack) {
			let mut pos = self.cursor;
			pos.skip(&haystack[..offset]);
			Some(pos)
		} else {
			None
		}
	}

	pub fn find_str_in_chunk(&mut self, needle: &str) -> Option<Pos> {
		self.find_in_chunk(|s: &str| s.find(needle))
	}

	pub fn find_char_in_chunk<T>(&mut self, search: T) -> Option<Pos>
	where
		T: Fn(char) -> bool,
	{
		self.find_in_chunk(|s: &str| s.find(&search))
	}
}

impl<'a> fmt::Debug for SpanIter<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		const MAX_TEXT_LEN: usize = 16;
		let text = if self.pending.len() > 0 {
			self.pending
		} else {
			&self.span.buffer[self.cursor.offset..]
		};
		write!(f, "Iter({:?} {} {}", self.cursor, self.span.indent, self.span.quotes)?;
		if text.len() <= MAX_TEXT_LEN {
			write!(f, " {:?}", text)?;
		} else {
			write!(f, " {:?}…", &text[..MAX_TEXT_LEN])?;
		}
		write!(f, ")")
	}
}

impl<'a> Iterator for SpanIter<'a> {
	type Item = &'a str;

	fn next(&mut self) -> Option<Self::Item> {
		let next = self.chunk();
		if next.len() > 0 {
			self.skip_chunk();
			Some(next)
		} else {
			None
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_markdown_span_iter() {
		fn check(indent: usize, quotes: usize, source: &str, expected: Vec<&str>) {
			let sta = Pos::default();
			let end = {
				let mut p = Pos::default();
				p.skip(source);
				p
			};
			let mut span = Span::new(source, sta, end);
			span.indent = indent;
			span.quotes = quotes;
			let iter = span.iter();
			let actual: Vec<_> = iter.collect();
			assert_eq!(expected, actual);
			for it in actual.iter() {
				if it.trim().len() > 0 {
					assert!(!span.text_range(it).is_none(), "chunk {:?} does not belong to span", it);
				}
			}
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

		// Multi-line iteration:
		check(
			1,
			1,
			"ABC\n\nDEF\r\r123\r\n\r\n456\n\n",
			vec!["ABC\n", "\n", "DEF\r", "\r", "123\r\n", "\r\n", "456\n", "\n"],
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

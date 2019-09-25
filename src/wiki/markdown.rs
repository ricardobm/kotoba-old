use std::collections::VecDeque;
use std::fmt;
use std::fmt::Write;

/// Default tab width for markdown text.
const TAB_WIDTH: usize = 4;

use regex::Regex;

use util;

/// Parse the input string as markdown, returning an iterator of [Element].
pub fn parse_markdown<'a>(input: &'a str) -> MarkdownIterator<'a> {
	MarkdownIterator::new(input)
}

/// Generate HTML code from the iterator returned by [parse_markdown].
pub fn to_html<'a>(iter: MarkdownIterator<'a>) -> util::Result<String> {
	let mut output = String::new();
	for it in iter {
		match it {
			Element::Break => output.push_str("<hr/>"),
			Element::Header(level, text) => {
				write!(output, "<h{0}>{1}</h{0}>", level, text)?;
			}
			Element::Paragraph(text) => {
				write!(output, "<p>{}</p>", text)?;
			}
			_ => panic!("{:?}", it),
		}
	}
	Ok(output)
}

/// Markdown block elements.
///
/// These elements are generated by the parser in order, as they are encountered
/// in the document.
#[derive(Clone, Debug)]
pub enum Element<'a> {
	// Leaf blocks:
	Break,
	Header(u8, Span<'a>),
	IndentedCode(Span<'a>),
	FencedCode {
		lang: &'a str,
		info: &'a str,
		code: Span<'a>,
	},
	Html(Span<'a>),

	LinkReference {
		uri:   RawStr<'a>,
		title: RawStr<'a>,
		label: Span<'a>,
	},

	Paragraph(Span<'a>),

	// Tables:
	TableSta,
	TableEnd,
	TableHeaderSta,
	TableHeaderEnd,
	TableBodySta,
	TableBodyEnd,
	TableRowSta,
	TableRowEnd,
	TableHead(Span<'a>),
	TableData(Span<'a>),

	// Container blocks
	QuoteSta,
	QuoteEnd,
	ListSta {
		ordered: bool,
	},
	ListEnd {
		ordered: bool,
	},
	ListItemSta {
		index:   usize,
		ordered: bool,
		task:    Option<bool>,
	},
	ListItemEnd {
		index:   usize,
		ordered: bool,
	},
}

/// Inline markdown elements. Those can be parsed from a [Span].
#[derive(Clone, Debug)]
pub enum Inline<'a> {
	/// Literal text.
	Text(&'a str),
	/// Inline links.
	Link {
		uri:   RawStr<'a>,
		title: RawStr<'a>,
		text:  Span<'a>,
	},
	/// Inline images.
	Image {
		uri:   RawStr<'a>,
		title: RawStr<'a>,
		text:  Span<'a>,
	},
	/// Forced line break.
	LineBreak,
	/// Emphasized text start.
	EmphasisSta,
	/// Emphasized text end.
	EmphasisEnd,
	/// Strong text start.
	StrongSta,
	/// Strong text end.
	StrongEnd,
	/// Strike-through (deleted) text start.
	StrikeThroughSta,
	/// Strike-through (deleted) text end.
	StrikeThroughEnd,
	/// Inline code text start.
	CodeSta,
	/// Inline code text end.
	CodeEnd,
}

/// Raw text from a markdown document that may contain unparsed escape
/// sequences.
#[derive(Clone, Debug)]
pub struct RawStr<'a>(&'a str);

impl<'a> fmt::Display for RawStr<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let RawStr(s) = self;
		write!(f, "{}", s)
	}
}

/// Raw text from a markdown code section supporting indented code blocks.
#[derive(Clone, Debug)]
pub struct CodeStr<'a> {
	/// Raw text for the code block.
	text: &'a str,
	/// Base indentation level for the code.
	base: &'a str,
}

/// Block of inline markdown text that can be independently parsed into [Inline]
/// elements.
#[derive(Clone, Debug)]
pub struct Span<'a> {
	/// Unparsed source text.
	///
	/// Note that this may contain quote markers and indentation that need to be
	/// skipped when parsing and escape characters that need to be translated.
	source: &'a str,
	/// Indicates the the text is to be considered as raw text (e.g. code and
	/// HTML blocks) instead of being parsed as markdown.
	is_raw: bool,
	/// Maximum base indentation to be removed from raw text.
	indent: usize,
	/// Number of quotation levels (e.g. `>`) to remove from text.
	quotes: usize,
	/// Whole buffer.
	buffer: &'a str,
}

impl<'a> Span<'a> {
	fn new(source: &'a str, buffer: &'a str) -> Span<'a> {
		Span {
			source: source,
			is_raw: false,
			indent: 0,
			quotes: 0,
			buffer: buffer,
		}
	}

	fn iter(&self) -> SpanIter<'a> {
		SpanIter {
			source: self.source,
			column: 0,
			indent: self.indent,
			quotes: self.quotes,
		}
	}

	fn append_text(&mut self, text: &'a str) {
		// This code assumes that both `self.source` and `text` are slices
		// of `buffer`, except when they are empty.
		if text.len() > 0 {
			let new_sta = text.as_ptr();
			let new_end = text[text.len()..].as_ptr();
			let buf_sta = self.buffer.as_ptr();
			let buf_end = self.buffer[self.buffer.len()..].as_ptr();

			assert!(new_sta >= buf_sta);
			assert!(new_end >= buf_sta);
			assert!(new_sta <= buf_end);
			assert!(new_end <= buf_end);

			if self.source.len() == 0 {
				self.source = text;
			} else {
				let sta = self.source.as_ptr();
				let end = self.source[self.source.len()..].as_ptr();
				assert!(new_sta >= end);
				assert!(new_end >= end);
				let sta_offset = (sta as usize) - (buf_sta as usize);
				let end_offset = (new_end as usize) - (buf_sta as usize);
				self.source = &self.buffer[sta_offset..end_offset];
			}
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
					let (text, col) = trim_start(source, column);
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
						let tw = TAB_WIDTH - (column % TAB_WIDTH);
						source = skip_chars(source, 1);
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
							source = skip_chars(source, 1);
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

impl<'a> fmt::Display for Span<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		// TODO: parse inlines
		for s in self.iter() {
			write!(f, "{}", s)?;
		}
		Ok(())
	}
}

/// Iterates over [Element]s in a markdown text.
pub struct MarkdownIterator<'a> {
	buffer: &'a str,
	offset: usize,
	blocks: VecDeque<Element<'a>>,
	queued: VecDeque<Element<'a>>,
}

impl<'a> MarkdownIterator<'a> {
	fn new(input: &'a str) -> MarkdownIterator<'a> {
		MarkdownIterator {
			buffer: input,
			offset: 0,
			blocks: Default::default(),
			queued: Default::default(),
		}
	}

	fn get_next(&mut self) -> Option<Element<'a>> {
		loop {
			if self.queued.len() == 0 && !self.at_end() {
				let line = self.read_line();
				self.push_line(line);
			}

			if let Some(item) = self.queued.pop_front() {
				break Some(item);
			}

			if self.at_end() {
				if self.blocks.len() > 0 {
					let closed = self.blocks.pop_back().unwrap();
					let closed = closed.close_elem();
					break Some(closed);
				}
				break None;
			}
		}
	}

	fn push_line(&mut self, mut text: &'a str) {
		let buffer = self.buffer;

		// check all open blocks for which ones can remain open given the line
		let mut open = 0;
		for i in 0..self.blocks.len() {
			if let Some(next_text) = self.blocks[i].can_continue(text) {
				text = next_text;
				open += 1;
			} else {
				break;
			}
		}

		let mut empty_line = text.trim().len() == 0;
		let next_block = self.parse_block_start(text, buffer);

		// close all unmatched blocks when opening a new block or if the line
		// is empty
		if empty_line || next_block.is_some() {
			while self.blocks.len() > open {
				let closed = self.blocks.pop_back().unwrap();
				let closed = closed.close_elem();
				self.queued.push_back(closed);
			}
		}

		// open new blocks
		if let Some((next_text, elem)) = next_block {
			text = next_text;
			empty_line = false;

			if let Some(elem) = elem {
				if !elem.is_leaf() {
					self.queued.push_back(elem.clone());
				}
				self.blocks.push_back(elem);
			}

			while let Some((next_text, elem)) = self.parse_block_start(text, buffer) {
				text = next_text;
				if let Some(elem) = elem {
					if !elem.is_leaf() {
						self.queued.push_back(elem.clone());
					}
					self.blocks.push_back(elem);
				}
			}
		}

		// Append text to the current open block.
		if !empty_line {
			if self.blocks.len() == 0 || !self.blocks[self.blocks.len() - 1].can_append_text() {
				if self.blocks.len() > 0 && self.blocks[self.blocks.len() - 1].is_leaf() {
					self.queued.push_back(self.blocks.pop_back().unwrap());
				}
				if text.trim().len() > 0 {
					let num_quotes = self
						.blocks
						.iter()
						.filter(|x| if let Element::QuoteSta = x { true } else { false })
						.count();
					self.blocks.push_back(Element::Paragraph(Span {
						source: text,
						is_raw: false,
						indent: 0,
						quotes: num_quotes,
						buffer: self.buffer,
					}))
				}
			} else {
				let len = self.blocks.len();
				if let Some(new_elem) = self.blocks[len - 1].append_text(text) {
					self.blocks[len - 1] = new_elem;
				}
			}
		}
	}

	#[inline]
	fn at_end(&mut self) -> bool {
		self.offset >= self.buffer.len()
	}

	/// Offset for the next line.
	fn read_line(&mut self) -> &'a str {
		// find the next `\n` or `\r` from the offset position
		let offset = self.offset;
		let text = &self.buffer[offset..];
		let mut iter = text.char_indices();
		let mut pos_eol = None;
		while let Some((_, ch)) = iter.next() {
			if ch == '\r' || ch == '\n' {
				// we want the offset of the character after the line break
				let mut next = iter.next();
				if ch == '\r' {
					if let Some((_, '\n')) = next {
						next = iter.next();
					}
				}
				let eol_offset = next.unwrap_or((text.len(), ' ')).0;
				pos_eol = Some(self.offset + eol_offset);
				break;
			}
		}

		let pos_eol = match pos_eol {
			Some(pos) => pos,
			None => {
				// we found the end of input before a line break
				self.buffer.len()
			}
		};

		self.offset = pos_eol;
		&self.buffer[offset..pos_eol]
	}

	fn parse_block_start(&mut self, line: &'a str, buffer: &'a str) -> Option<(&'a str, Option<Element<'a>>)> {
		lazy_static! {
			static ref RE_BREAK: Regex = Regex::new(r"^[ ]{0,3}([-_*]\s*){3,}\s*$").unwrap();
			static ref RE_HEADER: Regex =
				Regex::new(r"^(?P<s>[ ]{0,3}(?P<h>[#]{1,6}))(\s.*?)??(?P<e>(\s#+)?\s*)$").unwrap();
			static ref RE_SETEXT_HEADER: Regex = Regex::new(r"^[ ]{0,3}([-]{3,}|[=]{3,})\s*$").unwrap();
		}

		let trim_line = line.trim_end();
		let is_paragraph = if let Some(Element::Paragraph(..)) = self.blocks.iter().last() {
			true
		} else {
			false
		};
		if trim_line.len() == 0 {
			None
		} else if is_paragraph && RE_SETEXT_HEADER.is_match(trim_line) {
			let level = if line.trim().chars().next().unwrap() == '=' {
				1
			} else {
				2
			};
			if let Some(Element::Paragraph(span)) = self.blocks.pop_back() {
				self.blocks.push_back(Element::Header(level, span));
			} else {
				unreachable!();
			}
			Some(("", None))
		} else if RE_BREAK.is_match(trim_line) {
			// Semantic break
			Some(("", Some(Element::Break)))
		} else if let Some(caps) = RE_HEADER.captures(trim_line) {
			// ATX Heading
			let lvl = caps.name("h").unwrap().as_str().len() as u8;
			let sta = caps.name("s").unwrap().end();
			let end = caps.name("e").unwrap().start();
			let txt = &trim_line[sta..end];
			let elem = Element::Header(lvl, Span::new(txt.trim(), buffer));
			Some(("", Some(elem)))
		} else {
			None
		}
	}
}

impl<'a> Iterator for MarkdownIterator<'a> {
	type Item = Element<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.get_next()
	}
}

impl<'a> Element<'a> {
	fn is_leaf(&self) -> bool {
		match self {
			Element::Break => true,
			Element::Header(..) => true,
			Element::IndentedCode(..) => true,
			Element::FencedCode { .. } => true,
			Element::Html(..) => true,
			Element::LinkReference { .. } => true,
			Element::Paragraph(..) => true,

			// Tables
			Element::TableSta => false,
			Element::TableEnd => false,
			Element::TableHeaderSta => false,
			Element::TableHeaderEnd => false,
			Element::TableBodySta => false,
			Element::TableBodyEnd => false,
			Element::TableRowSta => false,
			Element::TableRowEnd => false,
			Element::TableHead(..) => true,
			Element::TableData(..) => true,

			// Container blocks
			Element::QuoteSta => false,
			Element::QuoteEnd => false,
			Element::ListSta { .. } => false,
			Element::ListEnd { .. } => false,
			Element::ListItemSta { .. } => false,
			Element::ListItemEnd { .. } => false,
		}
	}

	fn can_continue(&self, _line: &'a str) -> Option<&'a str> {
		None
	}

	fn can_append_text(&self) -> bool {
		match self {
			// Those have text but cannot be appended due to the way
			// they are parsed:

			// Header is either a single line (no append) or detected
			// after the complete text has already been collected.
			Element::Header(..) => false,
			// LinkReference is parsed from a complete paragraph.
			Element::LinkReference { .. } => false,

			// Those can be appended:
			Element::IndentedCode(..) => true,
			Element::FencedCode { .. } => true,
			Element::Html(..) => true,
			Element::Paragraph(..) => true,

			// Anything else has no text
			_ => false,
		}
	}

	fn append_text(&mut self, text: &'a str) -> Option<Element<'a>> {
		match self {
			Element::IndentedCode(span) => {
				span.append_text(text);
				None
			}
			Element::FencedCode { code, .. } => {
				code.append_text(text);
				None
			}
			Element::Html(span) => {
				span.append_text(text);
				None
			}
			Element::Paragraph(span) => {
				span.append_text(text);
				None
			}
			_ => panic!("invalid append_text: {:?}", self),
		}
	}

	fn close_elem(self) -> Element<'a> {
		match self {
			Element::Break => self,
			Element::Paragraph(mut span) => {
				span.source = span.source.trim();
				Element::Paragraph(span)
			}
			Element::Header(level, mut span) => {
				span.source = span.source.trim();
				Element::Header(level, span)
			}
			_ => panic!("invalid close_elem: {:?}", self),
		}
	}
}

type Range = std::ops::Range<usize>;

//
// String helpers
//

fn trim_start(s: &str, mut column: usize) -> (&str, usize) {
	let mut chars = s.char_indices();
	let mut index = s.len();
	while let Some((chr_index, chr)) = chars.next() {
		if chr.is_whitespace() {
			column = if chr == '\t' { tab(column) } else { column + 1 };
		} else {
			index = chr_index;
			break;
		}
	}
	(&s[index..], column)
}

fn indent_width(s: &str) -> usize {
	let mut width = 0;
	for chr in s.chars() {
		if chr == '\t' {
			width = tab(width);
		} else if chr.is_whitespace() {
			width += 1;
		} else {
			break;
		}
	}
	width
}

/// Skip characters from the string slice.
fn skip_chars(s: &str, n: usize) -> &str {
	&s[s.char_indices().skip(n).map(|x| x.0).next().unwrap_or(s.len())..]
}

/// Compute the next column number for a space character at the current
/// position.
#[inline(always)]
fn spc(column: usize, chr: char) -> Option<usize> {
	if chr == '\t' {
		Some(tab(column))
	} else if chr.is_whitespace() {
		Some(column + 1)
	} else {
		None
	}
}

/// Compute the next stop column for a tab at the current position.
#[inline(always)]
fn tab(column: usize) -> usize {
	column + (TAB_WIDTH - (column % TAB_WIDTH))
}

//
// TESTS
//

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

	#[test]
	fn test_markdown_simple() {
		// Simple paragraphs
		test(
			r"
			Paragraph 1

			Paragraph 2

			3.1
			3.2
			",
			vec!["<p>Paragraph 1</p>", "<p>Paragraph 2</p>", "<p>3.1\n3.2</p>"],
		);
	}

	#[test]
	fn test_markdown_breaks() {
		// Thematic breaks
		test(
			r"
			***

			---
			___

			P1

			   ---
			   ***
			   ___
			P2

			+++
			--
			**
			__

			 ***
			  * * *
			   *  *  *
			",
			vec![
				"<hr/><hr/><hr/>",
				"<p>P1</p>",
				"<hr/><hr/><hr/>",
				"<p>P2</p>",
				"<p>+++\n--\n**\n__</p>",
				"<hr/><hr/><hr/>",
			],
		);
	}

	#[test]
	fn test_markdown_atx_headings() {
		test(
			r"
			# H 1
			## H 2
			### H 3
			#### H 4
			##### H 5
			###### H 6

			P1
			 # H1 # ##############
			  ## H2##
			   ### H3 # # #
			P2
			####### H7
			",
			vec![
				"<h1>H 1</h1>",
				"<h2>H 2</h2>",
				"<h3>H 3</h3>",
				"<h4>H 4</h4>",
				"<h5>H 5</h5>",
				"<h6>H 6</h6>",
				"<p>P1</p>",
				"<h1>H1 #</h1>",
				"<h2>H2##</h2>",
				"<h3>H3 # #</h3>",
				"<p>P2\n####### H7</p>",
			],
		)
	}

	#[test]
	fn test_markdown_setext_headings() {
		test(
			r"
			Title 1
			=======

			Title 2
			-------

			Multi-line
			   Title 2
			   ---

			L1
			L2
			==
			===
			L3
			--
			---

			",
			vec![
				"<h1>Title 1</h1>",
				"<h2>Title 2</h2>",
				"<h2>Multi-line\n   Title 2</h2>",
				"<h1>L1\nL2\n==</h1>",
				"<h2>L3\n--</h2>",
			],
		);
	}

	fn test(input: &str, expected: Vec<&'static str>) {
		lazy_static! {
			static ref RE_INDENT: Regex = Regex::new(r"^\s*").unwrap();
		}

		let mut base_indent = "";
		let mut input_text = String::new();
		let mut has_indent = false;
		for (i, line) in input.trim().lines().enumerate() {
			if !has_indent && i > 0 && line.trim().len() > 0 {
				base_indent = RE_INDENT.find(line).unwrap().as_str();
				has_indent = true;
			}

			let line = if line.starts_with(base_indent) {
				&line[base_indent.len()..]
			} else {
				line
			};
			if i > 0 {
				input_text.push('\n');
			}
			input_text.push_str(line);
		}

		let result = to_html(parse_markdown(input_text.as_str())).unwrap();
		assert_eq!(result, expected.join(""));
	}
}

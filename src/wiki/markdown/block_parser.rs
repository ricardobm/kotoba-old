//! Event based parsing for Markdown blocks.
//!
//! Supports the CommonMark spec with GitHub extensions. This only implements
//! the block level parsing of the spec (i.e. no inlines).

use std::collections::VecDeque;
use std::fmt;

use regex::Regex;

use super::common;
use super::Span;

use super::table_parser::{parse_table_row, Row, TableRow};

use super::{Pos, TextBuffer};

pub fn parse_blocks<'a>(input: &'a str) -> BlockIterator<'a> {
	BlockIterator::new(input)
}

dbg_flag!(false);

/// Events generated by the block parser.
///
/// The events generated by the parser are designed in such a way to allow the
/// construction of the Markdown document tree.
///
/// Note that the events do not correspond 1-to-1 with Markdown elements, since
/// that requires some further processing and state keeping (particularly for
/// lists).
#[derive(Clone)]
pub enum BlockEvent<'a> {
	/// Event generated at the opening of a container block.
	Start(Container),
	/// Event generated at the closing of a container block.
	End(Container),
	/// Event generated for a leaf block element.
	Leaf(Leaf<'a>),
}

impl<'a> fmt::Debug for BlockEvent<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			BlockEvent::Start(elem) => {
				write!(f, "<{}", elem.tag())?;
				elem.fmt_attrs(f)?;
				write!(f, ">")?;
				Ok(())
			}
			BlockEvent::End(elem) => write!(f, "</{}>", elem.tag()),
			BlockEvent::Leaf(elem) => write!(f, "{:?}", elem),
		}
	}
}

/// Information for a list.
#[derive(Clone)]
pub struct ListInfo {
	/// For ordered lists this contains the start index. This is `None` for
	/// unordered lists.
	pub ordered: Option<usize>,
	/// The character that was used to mark this list item.
	pub marker: char,
	/// Relative indentation of the content of this list item.
	pub text_indent: usize,
	/// Indentation of the list marker.
	pub base_indent: usize,
	/// Contains the task state if this is a task item.
	pub task: Option<bool>,
	/// Position of the marker.
	pub marker_pos: Pos,
}

impl fmt::Debug for ListInfo {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "(`")?;
		if let Some(start) = self.ordered {
			write!(f, "{}", start)?;
		}
		write!(
			f,
			"{}`{} text={} base={})",
			self.marker,
			if let Some(task) = self.task {
				if task {
					" task=true"
				} else {
					" task=false"
				}
			} else {
				""
			},
			self.text_indent,
			self.base_indent,
		)
	}
}

impl ListInfo {
	pub fn is_next_same_list(&self, next: &ListInfo) -> bool {
		if self.ordered.is_some() != next.ordered.is_some() {
			false
		} else if self.marker != next.marker {
			false
		} else if next.base_indent >= 2 {
			false
		} else {
			true
		}
	}
}

/// Container blocks.
#[derive(Clone)]
pub enum Container {
	/// Markdown blockquote block.
	BlockQuote(Pos),
	/// List item.
	ListItem(ListInfo),
}

impl fmt::Debug for Container {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Container::BlockQuote(..) => write!(f, "BlockQuote"),
			Container::ListItem(info) => write!(f, "ListItem{:?}", info),
		}
	}
}

#[derive(Debug)]
enum CanContinue {
	No,
	Yes { position: Pos },
}

impl Container {
	/// HTML tag name for this container.
	fn tag(&self) -> &'static str {
		match self {
			Container::BlockQuote(..) => "blockquote",
			Container::ListItem(..) => "li",
		}
	}

	/// Output this container's tag attributes.
	fn fmt_attrs(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			&Container::BlockQuote(..) => Ok(()),
			&Container::ListItem(ListInfo {
				base_indent,
				ordered,
				task,
				..
			}) => {
				if base_indent > 0 {
					write!(f, " indent=\"{}\"", base_indent)?;
				}
				if let Some(index) = ordered {
					if index != 1 {
						write!(f, " start=\"{}\"", index)?;
					}
				}
				if let Some(task) = task {
					write!(f, " task=\"{}\"", task)?;
				}
				Ok(())
			}
		}
	}

	/// Check if the prefix of `line` allow for the continuation of this
	/// container, skipping any markings.
	///
	/// Returns `true` if the container is continued, the number of bytes
	/// to skip and an optional target column.
	fn can_continue<'a>(&self, line: Span<'a>) -> CanContinue {
		match self {
			&Container::BlockQuote(..) => {
				let (indent, bytes) = common::indent_width(line.text(), line.start.column);
				let mut next = line.start;
				let line = line.text();
				next.skip(&line[..bytes]);
				if indent > 3 {
					CanContinue::No
				} else if line.starts_with("> ") {
					next.skip("> ");
					CanContinue::Yes { position: next }
				} else if line.starts_with(">\t") {
					next.skip(">");
					if common::tab_width(next.column) == 1 {
						next.skip("\t");
					} else {
						next.column += 1;
					}
					CanContinue::Yes { position: next }
				} else if line.starts_with(">") {
					next.skip(">");
					CanContinue::Yes { position: next }
				} else {
					CanContinue::No
				}
			}
			&Container::ListItem(ListInfo {
				base_indent,
				text_indent,
				..
			}) => {
				if line.text().trim().len() == 0 {
					// List item can continue through empty lines
					CanContinue::Yes { position: line.start }
				} else {
					// We consider the list item continued if we can skip all
					// of its indentation.
					let target_indent = base_indent + text_indent;
					let mut cursor = line.start;
					let mut line = line.text();
					let mut do_continue = true;
					let start_column = cursor.column;
					while do_continue {
						do_continue = false;
						if let Some(ch) = line.chars().next() {
							if ch == ' ' || ch == '\t' {
								let new_column = common::col(ch, cursor.column);
								if new_column - start_column <= target_indent {
									cursor.skip(&line[..1]);
									line = &line[1..];
									do_continue = cursor.column - start_column < target_indent;
								} else {
									if ch == '\t' {
										// advance column without really
										// consuming the tab to simulate
										// partially consuming it
										cursor.column = start_column + target_indent;
									}
								}
							}
						}
					}

					if cursor.column - start_column == target_indent {
						CanContinue::Yes { position: cursor }
					} else {
						CanContinue::No
					}
				}
			}
		}
	}
}

/// Leaf blocks.
#[derive(Clone)]
pub enum Leaf<'a> {
	/// A single paragraph of markdown text.
	Paragraph { text: Span<'a> },
	/// HTML code block.
	HTML {
		/// Span of HTML code.
		code: Span<'a>,
		/// End condition for the HTML block. If `None` the end condition is
		/// the empty line.
		end: Option<&'static str>,
	},
	/// Link reference definition.
	LinkReference {
		/// Link target.
		url: Span<'a>,
		/// Link label, not including the `[]` delimiters.
		label: Span<'a>,
		/// Link title, not including the quotes.
		title: Option<Span<'a>>,
	},
	/// Indented code block.
	IndentedCode {
		/// Raw code span.
		code: Span<'a>,
	},
	/// Fenced code block.
	FencedCode {
		/// The fence delimiter.
		fence: &'a str,
		/// Raw code span including the whole block.
		span: Span<'a>,
		/// Just the code portion.
		code: Span<'a>,
		/// If the info string starts with a language tag, this will be it.
		lang: Option<&'a str>,
		/// Remaining info string, after extracting the language tag.
		info: Option<&'a str>,
	},
	/// Thematic break.
	Break(Pos),
	/// Setext or ATX header event.
	///
	/// - For a Setext header, this will be generated right after the
	///   respective [Paragraph].
	/// - For ATX headers this will contain the inline text for the header.
	Header {
		/// Header level from 1 to 6.
		level: u8,
		/// Header text.
		text: Span<'a>,
	},
	/// Table element.
	Table {
		span: Span<'a>,
		head: Option<TableRow<'a>>,
		body: Vec<TableRow<'a>>,
		cols: Option<usize>,
	},
}

impl<'a> fmt::Debug for Leaf<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Leaf::Paragraph { text } => write!(f, "<p>{}</p>", text),
			Leaf::HTML { code, .. } => write!(f, "<html>\n{}\n</html>", code),
			Leaf::LinkReference { url, label, title } => write!(f, "<a href={:?} title={:?}>{}</a>", url, title, label),
			Leaf::IndentedCode { code } => write!(f, "<code>\n{}\n</code>", code),
			Leaf::FencedCode { code, lang, info, .. } => {
				write!(f, "<code lang={:?} info={:?}>\n{}\n</code>", lang, info, code)
			}
			Leaf::Break(..) => write!(f, "<hr/>"),
			Leaf::Header { level, text } => write!(f, "<h{0}>{1}</h{0}>", level, text),
			Leaf::Table { head, body, .. } => {
				write!(f, "<table>")?;
				if let Some(head) = head {
					write!(f, "\n  <tr>")?;
					for (th, align) in head.iter() {
						write!(f, "<th")?;
						align.fmt_attr(f)?;
						write!(f, ">{}</th>", th)?;
					}
					write!(f, "</tr>")?;
				}
				for row in body.iter() {
					write!(f, "\n  <tr>")?;
					for (td, align) in row.iter() {
						write!(f, "<td")?;
						align.fmt_attr(f)?;
						write!(f, ">{}</td>", td)?;
					}
					write!(f, "</tr>")?;
				}
				write!(f, "\n</table>")
			}
		}
	}
}

/// States used by [BlockIterator].
#[derive(Debug)]
enum IteratorState {
	None,
	Open(usize, Option<Container>),
	Text(usize),
	Empty(usize),
	End,
}

impl Default for IteratorState {
	fn default() -> Self {
		IteratorState::None
	}
}

#[derive(Debug)]
enum LeafState<'a> {
	Closed(Leaf<'a>),
	ClosedAndConsumed(Leaf<'a>),
	Open(Leaf<'a>),
}

/// Iterates over [Event]s in a markdown text.
pub struct BlockIterator<'a> {
	state:  IteratorState,
	buffer: TextBuffer<'a>,
	inline: Option<Leaf<'a>>,
	blocks: VecDeque<Container>,

	next_line: Option<(usize, usize)>,
	line_num:  usize,
}

lazy_static! {
	static ref RE_BREAK: Regex = Regex::new(
		r"(?x)
			^\s*
			(
				([-]\s*){3,} |
				([_]\s*){3,} |
				([*]\s*){3,}
			)
			\s*$"
	)
	.unwrap();
	static ref RE_ATX_HEADER: Regex = Regex::new(r"^(?P<s>\s*(?P<h>[#]{1,6}))(\s.*?)??(?P<e>(\s#+)?\s*)$").unwrap();
	static ref RE_CODE_FENCE: Regex = Regex::new(
		r"(?x)
		^\s*
		(
			### Delimiter: ~~~ ###
			(?P<d0>~{3,})            # Main delimiter
			(\s*(?P<w0>\w+)(\s|$))?  # Optional language
			(?P<i0>.*)               # Additional info
			|
			### Delimiter: ``` ###
			(?P<d1>`{3,})            # Main delimiter
			(\s*(?P<w1>\w+)(\s|$))?  # Optional language
			(?P<i1>[^`]*)            # Additional info
		)$"
	)
	.unwrap();
}

impl<'a> BlockIterator<'a> {
	pub fn new(input: &'a str) -> BlockIterator<'a> {
		BlockIterator {
			state:  IteratorState::None,
			buffer: TextBuffer::new(input),
			inline: None,
			blocks: Default::default(),

			next_line: Default::default(),
			line_num:  0,
		}
	}

	fn get_next(&mut self) -> Option<BlockEvent<'a>> {
		// Loop until a generating state is found. Returns the new state and
		// the next Event.
		let (state, next) = loop {
			dbg_print!(" STATE : {:?}", self.state);

			// Match the current state and return the next state.
			self.state = match std::mem::take(&mut self.state) {
				// Final state after the input has been consumed.
				IteratorState::End => {
					break (IteratorState::End, None);
				}

				// This is the state at the start of the input or at the start
				// of a new line.
				IteratorState::None => {
					if self.buffer.at_end() {
						if let Some(inline) = std::mem::take(&mut self.inline) {
							let inline = self.close_leaf(inline, true);
							break (IteratorState::None, Some(BlockEvent::Leaf(inline)));
						} else if self.blocks.len() > 0 {
							let closed = self.blocks.pop_back().unwrap();
							break (IteratorState::None, Some(BlockEvent::End(closed)));
						} else {
							IteratorState::End
						}
					} else {
						// Skip any markers for currently open blocks...
						let open_count = self.skip_opened();
						// and next check for blocks to open.
						IteratorState::Open(open_count, None)
					}
				}

				// State at the beginning of a line, after skipping current
				// block markers, when checking for new blocks to open.
				IteratorState::Open(opened, None) => {
					if let Some(elem) = self.parse_container_start() {
						IteratorState::Open(opened, Some(elem))
					} else {
						IteratorState::Text(opened)
					}
				}

				// State when a block open has been matched...
				IteratorState::Open(opened, Some(elem)) => {
					if let Some(inline) = std::mem::take(&mut self.inline) {
						// ...first generate any pending inline (e.g. paragraph)
						let inline = self.close_leaf(inline, false);
						break (IteratorState::Open(opened, Some(elem)), Some(BlockEvent::Leaf(inline)));
					} else if self.blocks.len() > opened {
						// ...then close any non-continued open blocks
						let closed = self.blocks.pop_back().unwrap();
						break (IteratorState::Open(opened, Some(elem)), Some(BlockEvent::End(closed)));
					} else {
						// ...finally generate the new block and go back to the
						// empty open state to continue checking for new blocks.
						self.blocks.push_back(elem.clone());
						let opened = self.blocks.len();
						break (IteratorState::Open(opened, None), Some(BlockEvent::Start(elem)));
					}
				}

				// State when a blank line is found.
				IteratorState::Empty(opened) => {
					if self.blocks.len() > opened {
						// ...then close any non-continued open blocks
						let closed = self.blocks.pop_back().unwrap();
						break (IteratorState::Empty(opened), Some(BlockEvent::End(closed)));
					} else {
						// ...finally just reset the state for the next line
						IteratorState::None
					}
				}

				// State when matching the text content of a line
				IteratorState::Text(opened) => {
					let mut cur_line = self.buffer.cur_line();
					cur_line.quotes = self.cur_quotes();

					let mut do_skip = true;
					let (next_state, elem) = if let Some(inline) = std::mem::take(&mut self.inline) {
						// If there is a current leaf block open, append text
						// to it...
						match self.append_leaf(inline, cur_line, opened) {
							LeafState::Open(leaf) => {
								self.inline = Some(leaf);
								(IteratorState::None, None)
							}
							LeafState::ClosedAndConsumed(leaf) => {
								// ...if that closed the block, generate it.
								let leaf = self.close_leaf(leaf, false);
								(IteratorState::None, Some(BlockEvent::Leaf(leaf)))
							}
							LeafState::Closed(leaf) => {
								do_skip = false;
								let leaf = self.close_leaf(leaf, false);
								(IteratorState::None, Some(BlockEvent::Leaf(leaf)))
							}
						}
					} else if cur_line.text().trim().len() == 0 {
						// If the line is empty, handle it
						(IteratorState::Empty(opened), None)
					} else {
						// Parse the line as a leaf block.
						match Self::parse_leaf(cur_line, false).unwrap() {
							// We need to handle semantic breaks exceptionally
							// because they can close block level items.
							LeafState::ClosedAndConsumed(Leaf::Break(pos)) => {
								if self.blocks.len() > opened {
									let closed = self.blocks.pop_back().unwrap();
									break (IteratorState::Text(opened), Some(BlockEvent::End(closed)));
								} else {
									let leaf = self.close_leaf(Leaf::Break(pos), false);
									(IteratorState::None, Some(BlockEvent::Leaf(leaf)))
								}
							}

							LeafState::Open(leaf) => {
								self.inline = Some(leaf);
								(IteratorState::None, None)
							}
							LeafState::ClosedAndConsumed(leaf) => {
								let leaf = self.close_leaf(leaf, false);
								(IteratorState::None, Some(BlockEvent::Leaf(leaf)))
							}
							LeafState::Closed(_) => unreachable!(),
						}
					};

					if do_skip {
						self.buffer.skip_line();
					}

					if let Some(elem) = elem {
						break (next_state, Some(elem));
					} else {
						next_state
					}
				}
			};
		};

		dbg_print!("OUTPUT : {:?}", next);

		self.state = state;
		next
	}

	/// Skip the continuation markers for the currently open blocks.
	///
	/// Returns the number of blocks that can continue open.
	fn skip_opened(&mut self) -> usize {
		for i in 0..self.blocks.len() {
			let line = self.buffer.cur_line();
			if let CanContinue::Yes { position } = self.blocks[i].can_continue(line) {
				self.buffer.skip_to(position);
			} else {
				return i;
			}
		}
		self.blocks.len()
	}

	/// Return the number of open blockquotes.
	fn cur_quotes(&self) -> usize {
		(self.blocks)
			.iter()
			.filter(|x| if let Container::BlockQuote(..) = x { true } else { false })
			.count()
	}

	fn is_list_start(&mut self) -> bool {
		let line = self.buffer.cur_line().text();
		if line.starts_with(|ch| ch == '-' || ch == '+' || ch == '*') {
			!RE_BREAK.is_match(line)
		} else {
			false
		}
	}

	/// Parse opening block markers at the current position.
	fn parse_container_start(&mut self) -> Option<Container> {
		// Save current state in case we fail.
		let start_pos = self.buffer.save();

		match &self.inline {
			Some(Leaf::FencedCode { .. }) => return None,
			_ => {}
		}

		// Parse next block.
		let result = {
			// Skip optional indentation before the element
			let base_indent = self.buffer.skip_indent();
			let base_column = self.buffer.column();
			let base_pos = self.buffer.position();
			if base_indent > 3 {
				// We can have at most 3 spaces before becoming an indented
				// code block.
				None
			} else if self.buffer.skip_if('>') {
				// ----------
				// Blockquote
				// ----------

				// Skip one optional space after the marker.
				self.buffer.skip_indent_width(1);
				Some(Container::BlockQuote(base_pos))
			} else if self.is_list_start() {
				// -----------------------
				// List marker (unordered)
				// -----------------------

				let marker = self.buffer.next_char();
				self.buffer.skip_chars(1);
				if self.buffer.skip_indent_width(1) == 0 {
					// At least one space is needed after the list marker
					None
				} else {
					// The actual indent also considers the list item itself.
					let text_indent = self.buffer.column() - base_column;
					let task = self.parse_list_task();
					let list_info = ListInfo {
						marker,
						text_indent,
						base_indent,
						task,

						marker_pos: base_pos,
						ordered: None,
					};
					Some(Container::ListItem(list_info))
				}
			} else if self.buffer.cur_text().starts_with(|ch: char| ch.is_ascii_digit()) {
				// ---------------------
				// List marker (ordered)
				// ---------------------

				// Note that the spec limits the list index to 9 digits to
				// prevent overflow in browsers.
				let mut index = self.buffer.next_char().to_digit(10).unwrap() as usize;
				self.buffer.skip_chars(1);
				for _ in 0..8 {
					if !self.buffer.at_end() {
						let next = self.buffer.next_char();
						if let Some(digit) = next.to_digit(10) {
							index = index * 10 + (digit as usize);
							self.buffer.skip_chars(1);
						} else {
							break;
						}
					}
				}
				if self.buffer.cur_text().starts_with(|ch| ch == '.' || ch == ')') {
					// From here, the parsing is the same as for the unordered
					// case.
					let marker = self.buffer.next_char();
					self.buffer.skip_chars(1);

					let text_indent = self.buffer.skip_indent();
					if text_indent == 0 {
						None
					} else {
						let text_indent = self.buffer.column() - base_column;
						let task = self.parse_list_task();
						let list_info = ListInfo {
							marker,
							text_indent,
							base_indent,
							task,

							marker_pos: base_pos,
							ordered: Some(index),
						};
						Some(Container::ListItem(list_info))
					}
				} else {
					None
				}
			} else {
				None
			}
		};

		// Restore parser state if we failed to match.
		if result.is_none() {
			self.buffer.restore(start_pos);
		}
		result
	}

	fn parse_list_task(&mut self) -> Option<bool> {
		let result = if self.buffer.cur_text().starts_with("[x]") || self.buffer.cur_text().starts_with("[X]") {
			Some(true)
		} else if self.buffer.cur_text().starts_with("[ ]") {
			Some(false)
		} else {
			None
		};
		let result = if let Some(checked) = result {
			let start_pos = self.buffer.save();
			self.buffer.skip_chars(3);
			if !self.buffer.skip_if(' ') && !self.buffer.skip_if('\t') {
				if self.buffer.cur_line().text().trim().len() > 0 {
					self.buffer.restore(start_pos);
					None
				} else {
					Some(checked)
				}
			} else {
				Some(checked)
			}
		} else {
			None
		};
		result
	}

	/// Parse a leaf block at the given [Span].
	///
	/// If `is_inline` is true, this will parse as the continuation of a
	/// paragraph.
	///
	/// Returns a [LeafState] for the leaf block, or `None` if `is_inline` is
	/// true and the [Span] just continues the paragraph.
	///
	/// ## NOTE
	/// This will never return `None` if `is_inline` is false.
	fn parse_leaf(mut span: Span<'a>, is_inline: bool) -> Option<LeafState<'a>> {
		let text = span.text();
		let (indent, _) = common::indent_width(text, span.start.column);
		if indent >= 4 {
			// ===================
			// Indented code block
			// ===================
			if !is_inline {
				span.indent = 4 + span.start.column;
				span.skip = true;
				let code_block = Leaf::IndentedCode { code: span };
				Some(LeafState::Open(code_block))
			} else {
				None
			}
		} else if RE_BREAK.is_match(text) {
			// ===================
			// Semantic break
			// ===================
			Some(LeafState::ClosedAndConsumed(Leaf::Break(span.start)))
		} else if let Some(caps) = RE_ATX_HEADER.captures(text) {
			// ===================
			// ATX Heading
			// ===================
			let lvl = caps.name("h").unwrap().as_str().len() as u8;
			let sta = caps.name("s").unwrap().end();
			let end = caps.name("e").unwrap().start();
			span.start.skip(&text[..sta]);
			span.end = span.start;
			span.end.skip(&text[sta..end]);
			span.indent = 0;
			span = span.trimmed();
			let leaf = Leaf::Header {
				level: lvl,
				text:  span,
			};
			Some(LeafState::ClosedAndConsumed(leaf))
		} else if let Some(caps) = RE_CODE_FENCE.captures(text) {
			// ===================
			// Fenced code block
			// ===================
			let fence = caps.name("d0").unwrap_or_else(|| caps.name("d1").unwrap()).as_str();
			let lang = caps.name("w0");
			let lang = if let Some(x) = lang { Some(x) } else { caps.name("w1") };
			let lang = if let Some(x) = lang { x.as_str() } else { "" };
			let lang = if lang.len() > 0 { Some(lang) } else { None };
			let info = caps.name("i0");
			let info = if let Some(x) = info { Some(x) } else { caps.name("i1") };
			let info = if let Some(x) = info { x.as_str().trim() } else { "" };
			let info = if info.len() > 0 { Some(info) } else { None };
			span.indent = indent;
			Some(LeafState::Open(Leaf::FencedCode {
				fence,
				lang,
				info,
				span: span.clone(),
				code: span,
			}))
		} else if let Some(end) = Self::match_html_start(text, is_inline) {
			// ===================
			// HTML block
			// ===================
			Some(LeafState::Open(Leaf::HTML {
				end:  if end.len() == 0 { None } else { Some(end) },
				code: span,
			}))
		} else if let Some(row) = parse_table_row(span.clone(), true) {
			// ===================
			// HTML block
			// ===================
			let table = match row {
				Row::Delimiter(count) => Leaf::Table {
					span: span,
					head: None,
					body: Vec::new(),
					cols: Some(count),
				},
				Row::Content(row) => Leaf::Table {
					span: span,
					head: Some(row),
					body: Vec::new(),
					cols: None,
				},
			};
			Some(LeafState::Open(table))
		} else {
			if !is_inline {
				Some(LeafState::Open(Leaf::Paragraph { text: span }))
			} else {
				None
			}
		}
	}

	fn append_leaf(&mut self, mut leaf: Leaf<'a>, line: Span<'a>, opened: usize) -> LeafState<'a> {
		lazy_static! {
			static ref RE_SETEXT_HEADER: Regex = Regex::new(r"^[ ]{0,3}([-]{1,}|[=]{1,})\s*$").unwrap();
		}

		let line_trim = line.trimmed();
		let empty = line_trim.text().len() == 0;
		let (indent, _) = common::indent_width(line.text(), line.start.column);
		match leaf {
			Leaf::Paragraph { mut text } => {
				// A setext headings cannot be interpreted as block constructs
				// other than paragraphs, so the following is OK
				//
				//     heading
				//     -------
				//
				//     > heading
				//     > -------
				//
				//     - heading
				//       -------
				//
				// while the following is not
				//
				//     > not a heading
				//     ---------------
				//
				//     - not a heading
				//     ---------------

				let can_be_setext = self.blocks.len() == opened;
				if empty {
					LeafState::Closed(Leaf::Paragraph { text })
				} else if can_be_setext && RE_SETEXT_HEADER.is_match(line.text()) {
					// re-interpret the paragraph as a Setext heading
					let level = if line.text().trim().chars().next().unwrap() == '=' {
						1
					} else {
						2
					};
					let header = Leaf::Header {
						level,
						text: text.trimmed(),
					};
					LeafState::ClosedAndConsumed(header)
				} else {
					if let Some(_) = Self::parse_leaf(line.clone(), true) {
						LeafState::Closed(Leaf::Paragraph { text })
					} else {
						text.end = line.end;
						LeafState::Open(Leaf::Paragraph { text })
					}
				}
			}
			Leaf::IndentedCode { ref mut code } => {
				if indent < 4 && !empty {
					LeafState::Closed(leaf)
				} else {
					if indent >= 4 && !empty {
						code.end = line.end;
					}
					LeafState::Open(leaf)
				}
			}
			Leaf::FencedCode {
				fence,
				mut span,
				mut code,
				lang,
				info,
			} => {
				if line_trim.len() == 0 && opened < self.blocks.len() {
					// close with the container
					code.end = line.start;
					LeafState::Closed(Leaf::FencedCode {
						fence,
						span,
						code,
						lang,
						info,
					})
				} else if indent < 4 {
					let delim = fence.chars().next().unwrap();
					let is_close = line_trim.text().starts_with(fence);
					let is_close = is_close && line_trim.text().chars().all(|ch| ch == delim);
					if code.start == span.start {
						// skip the first line break
						code.start = line.start;
					}
					if is_close {
						code.end = line.start;
						span.end = line.end;
						LeafState::ClosedAndConsumed(Leaf::FencedCode {
							fence,
							span,
							code,
							lang,
							info,
						})
					} else {
						code.end = line.end;
						span.end = line.end;
						LeafState::Open(Leaf::FencedCode {
							fence,
							span,
							code,
							lang,
							info,
						})
					}
				} else {
					code.end = line.end;
					span.end = line.end;
					LeafState::Open(Leaf::FencedCode {
						fence,
						span,
						code,
						lang,
						info,
					})
				}
			}
			Leaf::HTML { end, mut code } => {
				if let Some(end) = end {
					code.end = line.end;
					let html = Leaf::HTML { end: Some(end), code };
					if line_trim.text().contains(end) {
						LeafState::ClosedAndConsumed(html)
					} else {
						LeafState::Open(html)
					}
				} else {
					let html = Leaf::HTML { end: None, code };
					if empty {
						LeafState::Closed(html)
					} else {
						LeafState::Open(html)
					}
				}
			}

			Leaf::Table {
				mut span,
				head,
				mut body,
				mut cols,
			} => {
				let has_separator = cols.is_some();
				if let Some(row) = parse_table_row(line_trim, !has_separator) {
					let mut is_valid = true;
					match row {
						Row::Delimiter(count) => {
							if count == head.as_ref().unwrap().len() {
								cols = Some(count);
							} else {
								// delimiter line must match the number of
								// cells in the header
								is_valid = false;
							}
						}
						Row::Content(row) => {
							if has_separator {
								body.push(row);
							} else {
								// second line of the table must be a separator
								is_valid = false;
							}
						}
					}

					if is_valid {
						span.end = line.end;
						LeafState::Open(Leaf::Table { span, head, body, cols })
					} else {
						if empty {
							LeafState::Closed(Leaf::Paragraph { text: span })
						} else {
							span.end = line.end;
							LeafState::Open(Leaf::Paragraph { text: span })
						}
					}
				} else {
					if has_separator && (head.is_some() || body.len() > 0) {
						LeafState::Closed(Leaf::Table { span, head, body, cols })
					} else if empty {
						LeafState::Closed(Leaf::Paragraph { text: span })
					} else {
						span.end = line.end;
						LeafState::Open(Leaf::Paragraph { text: span })
					}
				}
			}

			// Those are closed as soon as they are parsed, so they will never
			// be appended to:
			Leaf::Break(..) | Leaf::Header { .. } => unreachable!(),

			// Those are parsed when closing, so they would not occur either.
			Leaf::LinkReference { .. } => unreachable!(),
		}
	}

	fn close_leaf(&mut self, mut leaf: Leaf<'a>, is_eof: bool) -> Leaf<'a> {
		if let Leaf::Paragraph { text } = leaf {
			super::parse_link_ref(text)
		} else if let Leaf::FencedCode {
			ref span, ref mut code, ..
		} = &mut leaf
		{
			if code.start == span.start {
				code.start = span.end
			}
			if is_eof {
				code.end = self.buffer.position();
			}
			leaf
		} else {
			leaf
		}
	}

	/// Check if the line contains
	fn match_html_start(line: &str, inline: bool) -> Option<&'static str> {
		lazy_static! {
			static ref TAGS: Vec<&'static str> = vec![
				"address",
				"article",
				"aside",
				"base",
				"basefont",
				"blockquote",
				"body",
				"caption",
				"center",
				"col",
				"colgroup",
				"dd",
				"details",
				"dialog",
				"dir",
				"div",
				"dl",
				"dt",
				"fieldset",
				"figcaption",
				"figure",
				"footer",
				"form",
				"frame",
				"frameset",
				"h1",
				"h2",
				"h3",
				"h4",
				"h5",
				"h6",
				"head",
				"header",
				"hr",
				"html",
				"iframe",
				"legend",
				"li",
				"link",
				"main",
				"menu",
				"menuitem",
				"nav",
				"noframes",
				"ol",
				"optgroup",
				"option",
				"p",
				"param",
				"section",
				"source",
				"summary",
				"table",
				"tbody",
				"td",
				"tfoot",
				"th",
				"thead",
				"title",
				"tr",
				"track",
				"ul",
			];
		}

		fn is_tag_end(text: &str, closed: bool) -> bool {
			text.trim().len() == 0
				|| text.starts_with(|ch| ch == ' ' || ch == '>')
				|| (closed && text.starts_with("/>"))
		}

		let text_trim = line.trim_start();
		if text_trim.starts_with('<') {
			if text_trim.starts_with("<!--") {
				return Some("-->");
			}
			if text_trim.starts_with("<?") {
				return Some("?>");
			}
			if text_trim.starts_with("<![CDATA[") {
				return Some("]]>");
			}

			if text_trim.len() > 2 && text_trim.starts_with("<!") {
				let ch = text_trim.as_bytes()[2] as char;
				if ch >= 'A' && ch <= 'Z' {
					return Some(">");
				}
			}

			let lc = text_trim.to_lowercase();

			let m = lc.trim_start_matches("<script");
			if m.len() < lc.len() && is_tag_end(m, false) {
				return Some("</script>");
			}
			let m = lc.trim_start_matches("<pre");
			if m.len() < lc.len() && is_tag_end(m, false) {
				return Some("</pre>");
			}
			let m = lc.trim_start_matches("<style");
			if m.len() < lc.len() && is_tag_end(m, false) {
				return Some("</style>");
			}

			let m = lc.trim_start_matches("<");
			if m.len() < lc.len() {
				let m = m.trim_start_matches("/");
				for s in TAGS.iter() {
					let n = m.trim_start_matches(s);
					if n.len() < m.len() && is_tag_end(n, true) {
						return Some("");
					}
				}
			}

			if !inline {
				lazy_static! {
					static ref RE_OPEN_OR_CLOSING_TAG: Regex = Regex::new(
						r#"(?ix)
							^<
							[a-z][-a-z0-9]*              # Tag name

							# Attributes
							(
								\s+[_:a-z][-a-z0-9._:]*  # Attribute name

								# Attribute value
								(
									\s*=\s*
									(
										[^\s"'=<>`]+     # Unquoted value
										|
										'[^']*'          # Single quoted value
										|
										"[^"]*"          # Double quoted value
									)
								)?
							)*

							\s* /?>
						"#
					)
					.unwrap();
				}

				if let Some(m) = RE_OPEN_OR_CLOSING_TAG.find(text_trim) {
					// Open or closing tag should be followed only by
					// whitespace and the end of line
					if text_trim[m.start()..].trim().len() == 0 {
						return Some("");
					}
				}
			}
		}
		None
	}
}

impl<'a> Iterator for BlockIterator<'a> {
	type Item = BlockEvent<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.get_next()
	}
}

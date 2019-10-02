use std::collections::VecDeque;

use regex::Regex;

use super::html::html_entity;
use super::links;
use super::{Pos, Range, RawStr, Span, SpanIter};

const REPLACEMENT_CHAR: char = '\u{FFFD}';

#[derive(Copy, Clone, Debug)]
pub enum TextOrChar<'a> {
	Text(&'a str),
	Char(char),
}

#[derive(Clone, Debug)]
pub enum InlineEvent<'a> {
	/// Raw text to be output.
	///
	/// This is HTML safe to the extent that the basic HTML entities generate
	/// an [Entity] event.
	Text(&'a str),
	/// A hard line break (e.g. `<br/>`).
	LineBreak,
	/// Generated either from HTML entities in the Markdown text or from
	/// raw characters that need HTML escaping (`<`, `>`, `&`, `\`, `"`, `'`).
	Entity {
		/// The source text.
		///
		/// This can be either an entity or a character that needs escaping.
		source: &'a str,
		/// The HTML entity to generate.
		entity: &'a str,
		/// The actual Unicode text corresponding to the entity.
		output: TextOrChar<'a>,
	},
	/// Generates a single character of text.
	Char(char),
	/// Either a `< >` delimited URL or a detected hyperlink.
	AutoLink {
		/// The matched link address, as it appears on the source text.
		///
		/// This should also be used as the link label. It may or may not
		/// contain the scheme.
		link: &'a str,
		/// Scheme prefix, excluding the `:`.
		///
		/// If the source does not contain the scheme, this will be a static
		/// string with the detected schema.
		scheme: &'a str,
		/// This will contain the necessary schema prefix in case the [link]
		/// does not contain it, being empty otherwise.
		///
		/// The prefix includes the `:` and possibly the `//`. It should not
		/// be used as part of the label.
		prefix: &'a str,
		/// True if this autolink was delimited by `< >`.
		delimited: bool,
	},
	/// Open an inline element.
	Open(Inline),
	/// Close an inline element.
	Close(Inline),
	/// A normal link.
	Link {
		url:   RawStr<'a>,
		label: Span<'a>,
		title: Span<'a>,
	},
	/// An image/media link.
	Image {
		url:   RawStr<'a>,
		label: Span<'a>,
		title: Span<'a>,
	},
	/// Raw HTML to be output verbatim.
	HTML {
		/// Only the tag name, as it appears on the source.
		tag: &'a str,
		/// Full HTML tag to be output verbatim.
		code: &'a str,
	},
}

#[derive(Clone, Debug)]
pub enum Inline {
	Code,
	Emphasis,
	Strong,
	Strikethrough,
}

#[derive(Clone)]
pub struct InlineIterator<'a> {
	block: Span<'a>,
	inner: SpanIter<'a>,
	queue: VecDeque<char>,
	state: State<'a>,

	next_link: Option<(Range, Option<(Range, InlineEvent<'a>)>)>,
}

impl<'a> Iterator for InlineIterator<'a> {
	type Item = InlineEvent<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.get_next()
	}
}

#[derive(Clone, Debug)]
enum State<'a> {
	Start,
	OutputNext(InlineEvent<'a>),
	OutputCodeText(Pos, &'a str, SpanIter<'a>),
	AutoLink(usize, InlineEvent<'a>),
	End,
}

impl<'a> Default for State<'a> {
	fn default() -> Self {
		State::Start
	}
}

impl<'a> InlineIterator<'a> {
	pub fn new(span: &Span<'a>) -> InlineIterator<'a> {
		InlineIterator {
			block: span.clone(),
			inner: span.iter(),
			queue: VecDeque::new(),
			state: State::Start,

			next_link: None,
		}
	}

	fn chunk(&mut self) -> &'a str {
		self.inner.chunk()
	}

	#[allow(unreachable_code)]
	fn get_next(&mut self) -> Option<InlineEvent<'a>> {
		let (next_state, result) = loop {
			self.state = match std::mem::take(&mut self.state) {
				State::End => break (State::End, None),

				State::OutputNext(event) => {
					break (State::Start, Some(event));
				}

				State::Start => {
					if !self.assert_chunk() {
						break (State::End, None);
					} else if let Some((Range { start: 0, end }, event)) = self.next_autolink() {
						self.skip_len(end);
						break (State::Start, Some(event));
					} else if let Some(index) = self.chunk().find(|c| is_special_char(c)) {
						let index = if let Some((link, _)) = self.next_autolink() {
							if link.start < index {
								link.start
							} else {
								index
							}
						} else {
							index
						};
						if index > 0 {
							let text = self.consume_and_skip(index);
							let event = InlineEvent::Text(text);
							break (State::Start, Some(event));
						}
						let next = self.next_char();
						match next {
							'\\' => {
								break self.parse_escape();
							}
							'&' => {
								break self.parse_entity();
							}
							'`' => {
								break self.parse_code();
							}
							'<' | '>' | '\'' | '"' | '\0' => {
								if next == '<' {
									if let Some(link) = links::parse_autolink(&mut self.inner) {
										break (State::Start, Some(link));
									}
								}
								let (len, event) = next_char_escaped(self.chunk());
								self.skip_len(len);
								break (State::Start, event);
							}
							_ => panic!("panicked at next char '{:?}'", next),
						};
					} else {
						// Detect GFM extension autolinks
						let len = if let Some((range, _)) = self.next_autolink() {
							debug_assert!(range.start > 0);
							range.start
						} else {
							self.chunk().len()
						};
						let text = self.consume_and_skip(len);
						let event = InlineEvent::Text(text);
						break (State::Start, Some(event));
					}
				}

				State::AutoLink(len, event) => {
					self.inner.skip_len(len);
					break (State::Start, Some(event));
				}

				State::OutputCodeText(end, delim, mut iter) => {
					if iter.at_end() {
						// at the end of the tag, generate the Close event and
						// consume the end delimiter
						self.inner.skip_to(end);
						self.skip_len(delim.len());
						break (State::Start, Some(InlineEvent::Close(Inline::Code)));
					}

					// Find the next character that needs escaping.
					if let Some(esc) = iter.find_char_in_chunk(|c| is_html_escaped(c) || c == '\r' || c == '\n') {
						let sta = iter.pos();
						if esc > sta {
							// generate text before the character
							iter.skip_to(esc);
							let text = self.block.sub_text(sta..esc);
							let next = State::OutputCodeText(end, delim, iter);
							let event = InlineEvent::Text(text);
							break (next, Some(event));
						} else {
							// generate the HTML escape or line break as space
							let chr = iter.chunk().chars().next().unwrap();
							let event = if chr == '\n' {
								iter.skip_len(1);
								Some(InlineEvent::Text(" "))
							} else if chr == '\r' {
								iter.skip_len(1);
								if iter.chunk().chars().next() == Some('\n') {
									iter.skip_len(1);
								}
								Some(InlineEvent::Text(" "))
							} else {
								Self::escape_next(&mut iter)
							};
							let next = State::OutputCodeText(end, delim, iter);
							break (next, event);
						}
					} else {
						// generate the whole text range
						let text = iter.chunk();
						iter.skip_chunk();
						let next = State::OutputCodeText(end, delim, iter);
						let event = InlineEvent::Text(text);
						break (next, Some(event));
					}
				}
			};
		};

		self.state = next_state;
		result
	}

	//
	// Parsing helpers
	//

	/// Find and parses the next autolink in the current chunk.
	fn next_autolink(&mut self) -> Option<(Range, InlineEvent<'a>)> {
		let cur_txt = self.inner.chunk();
		let cur_pos = self.inner.pos().offset;
		let cur_len = cur_txt.len();
		if let Some((valid_range, link)) = &self.next_link {
			if cur_pos >= valid_range.start && cur_pos < valid_range.end {
				if let Some((range, event)) = link {
					if cur_pos <= range.start {
						let out_range = Range {
							start: range.start - cur_pos,
							end:   range.end - cur_pos,
						};
						return Some((out_range, event.clone()));
					}
				} else {
					// cached range is valid, but it does not have a link
					return None;
				}
			}
		}

		// The range for which this cached value is valid
		let cur_range = Range {
			start: cur_pos,
			end:   cur_pos + cur_len,
		};
		if let Some((range, event)) = links::parse_autolink_extension(cur_txt) {
			// store the absolute range instead
			let abs_range = Range {
				start: range.start + cur_pos,
				end:   range.end + cur_pos,
			};
			self.next_link = Some((cur_range, Some((abs_range, event.clone()))));
			Some((range, event))
		} else {
			self.next_link = Some((cur_range, None));
			None
		}
	}

	fn escape_next(iter: &mut SpanIter<'a>) -> Option<InlineEvent<'a>> {
		let (len, event) = next_char_escaped(iter.chunk());
		iter.skip_len(len);
		event
	}

	fn parse_code(&mut self) -> (State<'a>, Option<InlineEvent<'a>>) {
		let parsed = super::inline_code::parse(&self.inner);
		let (code, end, delim) = (parsed.code, parsed.end, parsed.delim);
		if let Some(code) = code {
			let event = InlineEvent::Open(Inline::Code);
			(State::OutputCodeText(end, delim, code.iter()), Some(event))
		} else {
			// generate the delimiter as raw text
			let event = InlineEvent::Text(delim);
			self.skip_len(delim.len());
			(State::Start, Some(event))
		}
	}

	/// Parse an escape sequence at the backslash.
	fn parse_escape(&mut self) -> (State<'a>, Option<InlineEvent<'a>>) {
		if let (len, Some(escape)) = parse_escape(self.chunk()) {
			debug_assert!(len > 0);
			self.skip_len(len);
			(State::Start, Some(escape))
		} else {
			// non-recognized escape sequences are just generated literally
			let backslash = InlineEvent::Text(self.consume_and_skip(1));
			if let (len, Some(next_char)) = next_char_escaped(self.chunk()) {
				self.skip_len(len);
				(State::OutputNext(next_char), Some(backslash))
			} else {
				(State::Start, Some(backslash))
			}
		}
	}

	fn parse_entity(&mut self) -> (State<'a>, Option<InlineEvent<'a>>) {
		use super::entities::get_named_entity;

		lazy_static! {
			static ref RE_ENTITY: Regex = Regex::new(r#"^&\w+;"#).unwrap();
			static ref RE_ENTITY_DEC: Regex = Regex::new(r#"^&\#(?P<v>[0-9]{1,7});"#).unwrap();
			static ref RE_ENTITY_HEX: Regex = Regex::new(r#"^&\#[xX](?P<v>[0-9A-Fa-f]{1,6});"#).unwrap();
		}

		if let Some(m) = RE_ENTITY.find(self.chunk()) {
			let len = m.end();
			let entity = m.as_str();
			if let Some(output) = get_named_entity(entity) {
				let event = InlineEvent::Entity {
					source: entity,
					entity: entity,
					output: TextOrChar::Text(output),
				};
				self.skip_len(len);
				return (State::Start, Some(event));
			}
		} else if let Some(caps) = RE_ENTITY_DEC.captures(self.chunk()) {
			let src = caps.get(0).unwrap();
			let len = src.end();
			let src = src.as_str();
			let dec = caps.name("v").unwrap().as_str().parse::<u32>().unwrap();
			let chr = std::char::from_u32(dec).unwrap_or(REPLACEMENT_CHAR);
			let event = entity_or_char(src, chr);
			self.skip_len(len);
			return (State::Start, Some(event));
		} else if let Some(caps) = RE_ENTITY_HEX.captures(self.chunk()) {
			let src = caps.get(0).unwrap();
			let len = src.end();
			let src = src.as_str();
			let hex = u32::from_str_radix(caps.name("v").unwrap().as_str(), 16).unwrap();
			let chr = std::char::from_u32(hex).unwrap_or(REPLACEMENT_CHAR);
			let event = entity_or_char(src, chr);
			self.skip_len(len);
			return (State::Start, Some(event));
		}

		let (len, event) = next_char_escaped(self.chunk());
		self.skip_len(len);
		(State::Start, event)
	}

	//
	// Buffer reading
	//

	fn next_char(&mut self) -> char {
		self.chunk().chars().next().unwrap()
	}

	fn peek(&mut self, n: usize) -> Option<char> {
		while n >= self.queue.len() {
			if let Some(chr) = self.read_char() {
				self.queue.push_back(chr);
			} else {
				return None;
			}
		}
		Some(self.queue[n])
	}

	fn read_char(&mut self) -> Option<char> {
		if self.assert_chunk() {
			let mut chars = self.chunk().char_indices();
			if let Some((_, chr)) = chars.next() {
				let len = chars.next().map(|x| x.0).unwrap_or(self.chunk().len());
				self.inner.skip_len(len);
				Some(chr)
			} else {
				None
			}
		} else {
			None
		}
	}

	#[inline]
	fn skip_len(&mut self, len: usize) {
		self.inner.skip_len(len);
	}

	fn consume_and_skip(&mut self, len: usize) -> &'a str {
		if len > 0 {
			if !self.assert_chunk() {
				panic!("consume_and_skip({}) at the end of input", len);
			}
			let chunk = self.chunk();
			debug_assert!(chunk.len() >= len);
			self.inner.skip_len(len);
			&chunk[..len]
		} else {
			&self.block.text()[self.block.len()..]
		}
	}

	#[inline]
	fn assert_chunk(&mut self) -> bool {
		return self.chunk().len() > 0;
	}
}

//=====================================
// Helper functions
//=====================================

fn parse_escape<'a>(text: &'a str) -> (usize, Option<InlineEvent<'a>>) {
	lazy_static! {
		static ref RE_VALID_ESCAPE: Regex = Regex::new(
			r#"(?x)
				^\\[\-\\\{\}\[\]\(\)\^\|\~\&\$\#/:<>"!%'*+,.;=?@_`]
			"#
		)
		.unwrap();
		static ref RE_HARD_BREAK: Regex = Regex::new(
			r#"(?x)
				^\\[\s&&[^\n\r]]*(\n|\r\n?)
			"#
		)
		.unwrap();
	}

	if RE_VALID_ESCAPE.is_match(text) {
		let text = &text[1..];
		let (len, event) = next_char_escaped(text);
		debug_assert!(len > 0);
		(len + 1, event)
	} else if let Some(m) = RE_HARD_BREAK.find(text) {
		(m.end(), Some(InlineEvent::LineBreak))
	} else {
		(0, None)
	}
}

fn next_char_escaped<'a>(text: &'a str) -> (usize, Option<InlineEvent<'a>>) {
	let mut chars = text.char_indices();
	if let Some((_, chr)) = chars.next() {
		let len = chars.next().map(|x| x.0).unwrap_or(text.len());
		let txt = &text[0..len];
		let event = if let Some(entity) = html_entity(chr) {
			InlineEvent::Entity {
				source: txt,
				entity: entity,
				output: TextOrChar::Text(txt),
			}
		} else {
			InlineEvent::Text(txt)
		};
		(len, Some(event))
	} else {
		(0, None)
	}
}

fn entity_or_char<'a>(source: &'a str, c: char) -> InlineEvent<'a> {
	if let Some(entity) = html_entity(c) {
		InlineEvent::Entity {
			source: source,
			entity: entity,
			output: TextOrChar::Char(c),
		}
	} else {
		InlineEvent::Char(c)
	}
}

fn is_html_escaped(chr: char) -> bool {
	match chr {
		'\0' | '&' | '<' | '>' | '\'' | '"' => true,
		_ => false,
	}
}

/// Check if a character needs special handling as an inline.
#[inline]
fn is_special_char(chr: char) -> bool {
	match chr {
		// escapes
		'\\' => true,
		// should be replaced by `U+FFFD`
		'\0' => true,
		// HTML entities
		'&' | '<' | '>' | '\'' | '"' => true,
		// code spans
		'`' => true,
		// emphasis and strikethrough
		'*' | '_' | '~' => false,
		// links
		'[' | '!' => false,
		// line breaks
		'\n' | '\r' => false, // TODO: handle breaks
		_ => false,
	}
}

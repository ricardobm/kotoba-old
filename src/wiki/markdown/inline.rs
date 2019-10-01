use std::collections::VecDeque;

use regex::Regex;

use super::{RawStr, Span, SpanIter};

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
		/// The actual Unicode character corresponding to the entity.
		output: &'a str,
	},
	/// Either a `< >` delimited URL or a detected hyperlink.
	AutoLink {
		uri:       &'a str,
		scheme:    &'a str,
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
pub struct InlineIterator<'a, T: Iterator<Item = &'a str>> {
	inner: T,
	chunk: &'a str,
	queue: VecDeque<char>,
	state: State<'a>,
}

impl<'a, T: Iterator<Item = &'a str>> Iterator for InlineIterator<'a, T> {
	type Item = InlineEvent<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.get_next()
	}
}

#[derive(Clone, Debug)]
enum State<'a> {
	Start,
	OutputNext(InlineEvent<'a>),
	End,
}

impl<'a> Default for State<'a> {
	fn default() -> Self {
		State::Start
	}
}

impl<'a, T: Iterator<Item = &'a str>> InlineIterator<'a, T> {
	pub fn new(span: &Span<'a>) -> InlineIterator<'a, SpanIter<'a>> {
		InlineIterator {
			inner: span.iter(),
			chunk: "",
			queue: VecDeque::new(),
			state: State::Start,
		}
	}

	fn get_next(&mut self) -> Option<InlineEvent<'a>> {
		let (next_state, result) = loop {
			self.state = match std::mem::take(&mut self.state) {
				State::End => break (State::End, None),

				State::OutputNext(event) => {
					break (State::Start, Some(event));
				}

				State::Start => {
					if !self.assert_chunk() {
						State::End
					} else if let Some(index) = self.chunk.find(|c| is_special_char(c)) {
						if index > 0 {
							let text = self.consume_chunk(index);
							let event = InlineEvent::Text(text);
							break (State::Start, Some(event));
						}
						match self.next_char() {
							'\\' => {
								break self.parse_escape();
							}
							'&' => {
								break self.parse_entity();
							}
							'<' | '>' | '\'' | '"' => {
								let (len, event) = next_char_escaped(self.chunk);
								self.consume_chunk(len);
								break (State::Start, event);
							}
							_ => panic!("panicked at next char '{:?}'", self.next_char()),
						};
					} else {
						let text = self.consume_chunk(self.chunk.len());
						let event = InlineEvent::Text(text);
						break (State::Start, Some(event));
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

	/// Parse an escape sequence at the backslash.
	fn parse_escape(&mut self) -> (State<'a>, Option<InlineEvent<'a>>) {
		if let (len, Some(escape)) = parse_escape(self.chunk) {
			debug_assert!(len > 0);
			self.consume_chunk(len);
			(State::Start, Some(escape))
		} else {
			// non-recognized escape sequences are just generated literally
			let backslash = InlineEvent::Text(self.consume_chunk(1));
			if let (len, Some(next_char)) = next_char_escaped(self.chunk) {
				self.consume_chunk(len);
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
		}

		if let Some(m) = RE_ENTITY.find(self.chunk) {
			let len = m.end();
			let entity = m.as_str();
			if let Some(output) = get_named_entity(entity) {
				let event = InlineEvent::Entity {
					source: entity,
					entity: entity,
					output: output,
				};
				self.consume_chunk(len);
				return (State::Start, Some(event));
			}
		}

		let (len, event) = next_char_escaped(self.chunk);
		self.consume_chunk(len);
		(State::Start, event)
	}

	//
	// Buffer reading
	//

	fn next_char(&mut self) -> char {
		self.chunk.chars().next().unwrap()
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
			let mut chars = self.chunk.char_indices();
			if let Some((_, chr)) = chars.next() {
				let len = chars.next().map(|x| x.0).unwrap_or(self.chunk.len());
				self.chunk = &self.chunk[len..];
				Some(chr)
			} else {
				None
			}
		} else {
			None
		}
	}

	#[inline]
	fn consume_chunk(&mut self, len: usize) -> &'a str {
		debug_assert!(self.chunk.len() > 0 && len <= self.chunk.len() && len > 0);
		let chunk = &self.chunk[..len];
		self.chunk = &self.chunk[len..];
		chunk
	}

	#[inline]
	fn consume_chars(&mut self, count: usize) -> &'a str {
		let len = self
			.chunk
			.char_indices()
			.skip(count)
			.next()
			.map(|x| x.0)
			.unwrap_or(self.chunk.len());
		if len > 0 {
			self.consume_chunk(len)
		} else {
			&self.chunk[..0]
		}
	}

	#[inline]
	fn assert_chunk(&mut self) -> bool {
		if self.chunk.len() == 0 {
			if let Some(chunk) = self.inner.next() {
				self.chunk = chunk;
			}
		}
		return self.chunk.len() > 0;
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
		let entity = match chr {
			'"' => "&quot;",
			'&' => "&amp;",
			'<' => "&lt;",
			'>' => "&gt;",
			'\'' => "&apos;",
			_ => {
				return (len, Some(InlineEvent::Text(txt)));
			}
		};
		let event = InlineEvent::Entity {
			source: txt,
			entity: entity,
			output: txt,
		};
		(len, Some(event))
	} else {
		(0, None)
	}
}

/// Check if a character needs special handling as an inline.
#[inline]
fn is_special_char(chr: char) -> bool {
	match chr {
		// escapes
		'\\' => true,
		// HTML entities
		'&' | '<' | '>' | '\'' | '"' => true,
		// code spans
		'`' => false,
		// emphasis and strikethrough
		'*' | '_' | '~' => false,
		// links
		'[' | '!' => false,
		// line breaks
		'\n' | '\r' => false, // TODO: handle breaks
		_ => false,
	}
}

use super::{RawStr, Span, SpanIter};
use std::collections::VecDeque;

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
		entity: &'static str,
		/// The actual Unicode character corresponding to the entity.
		output: char,
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
	state: State,
}

impl<'a, T: Iterator<Item = &'a str>> Iterator for InlineIterator<'a, T> {
	type Item = InlineEvent<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.get_next()
	}
}

#[derive(Copy, Clone, Debug)]
enum State {
	Start,
	End,
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
			self.state = match self.state {
				State::End => break (State::End, None),

				State::Start => {
					if !self.assert_chunk() {
						State::End
					} else if let Some(index) = self.chunk.find(|c| is_special_char(c)) {
						if index > 0 {
							let text = self.consume_chunk(index);
							let event = InlineEvent::Text(text);
							break (State::Start, Some(event));
						}
						panic!("panicked at next char '{:?}'", self.chunk.chars().next());
					} else {
						let text = self.consume_chunk(0);
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
	// Buffer reading
	//

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
		debug_assert!(self.chunk.len() > 0 && len < self.chunk.len());
		let len = if len > 0 { len } else { self.chunk.len() };
		let chunk = &self.chunk[..len];
		self.chunk = &self.chunk[len..];
		chunk
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

/// Check if a character needs special handling as an inline.
#[inline]
fn is_special_char(chr: char) -> bool {
	// TODO: enable handling of those
	match chr {
		// escapes
		'\\' => false,
		// HTML entities
		'&' | '<' | '>' | '\'' | '"' => false,
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

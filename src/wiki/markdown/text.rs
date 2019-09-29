use std::fmt;

use super::common;
use super::Span;

#[derive(Copy, Clone, Default)]
pub struct Pos {
	pub line:   usize,
	pub column: usize,
	pub offset: usize,
	was_cr:     bool,
}

impl fmt::Debug for Pos {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "({}@L{}:{})", self.offset, self.line + 1, self.column + 1)
	}
}

impl Pos {
	pub fn advance(&self, buffer: &str, offset: usize) -> Pos {
		let mut out = *self;
		out.skip(&buffer[self.offset..self.offset + offset]);
		out
	}

	pub fn skip(&mut self, src: &str) {
		for c in src.chars() {
			self.do_skip_char(c);
		}
		self.offset += src.len();
	}

	pub fn skip_char(&mut self, c: char) {
		self.do_skip_char(c);
		self.offset += c.len_utf8();
	}

	#[inline(always)]
	fn do_skip_char(&mut self, c: char) {
		match c {
			'\r' => {
				self.column = 0;
				self.line += 1;
				self.was_cr = true;
			}
			'\n' => {
				if !self.was_cr {
					self.column = 0;
					self.line += 1;
				}
				self.was_cr = false;
			}
			'\t' => {
				self.column = common::tab(self.column);
				self.was_cr = false;
			}
			_ => {
				self.column += 1;
				self.was_cr = false;
			}
		}
	}
}

#[derive(Clone)]
pub struct TextBuffer<'a> {
	src: &'a str,
	pos: Pos,
	eol: Option<(Pos, usize)>,
	eof: Option<Pos>,
}

impl<'a> TextBuffer<'a> {
	pub fn new(source: &'a str) -> TextBuffer<'a> {
		TextBuffer {
			src: source,
			pos: Default::default(),
			eol: None,
			eof: None,
		}
	}

	pub fn column(&self) -> usize {
		self.pos.column
	}

	/// Return the text at the current position in the buffer.
	pub fn cur_text(&self) -> &'a str {
		&self.src[self.pos.offset..]
	}

	/// Return the state of the buffer that can be restored with [restore].
	pub fn save(&self) -> Pos {
		self.pos
	}

	/// Restore a state returned by [save].
	pub fn restore(&mut self, pos: Pos) {
		self.pos = pos;
	}

	/// Skip to a direct position.
	pub fn skip_to(&mut self, pos: Pos) {
		self.pos = pos;
	}

	/// Return the next charater at the current offset. Panics at the end of
	/// the input.
	#[inline(always)]
	pub fn next_char(&self) -> char {
		self.src[self.pos.offset..].chars().next().unwrap()
	}

	/// Skip the next char, only if it is equal to the given one.
	#[inline(always)]
	pub fn skip_if(&mut self, chr: char) -> bool {
		if self.src[self.pos.offset..].starts_with(chr) {
			self.skip_chars(1);
			true
		} else {
			false
		}
	}

	/// `true` when at the end of input.
	pub fn at_end(&self) -> bool {
		self.pos.offset >= self.src.len()
	}

	/// Skip indentation of the current line and return the indentation width.
	pub fn skip_indent(&mut self) -> usize {
		let (columns, len) = common::indent_width(&self.src[self.pos.offset..], self.pos.column);
		self.skip(len);
		columns
	}

	/// Skip the given number of bytes from the input.
	pub fn skip(&mut self, len: usize) {
		self.pos.skip(&self.src[self.pos.offset..self.pos.offset + len]);
	}

	pub fn skip_line(&mut self) {
		self.pos = self.eol_pos(true);
	}

	/// Skip the specified number of characters from the input.
	#[inline(always)]
	pub fn skip_chars(&mut self, n: usize) {
		let text_len = self.src.len() - self.pos.offset;
		let skip_len = (&self.src[self.pos.offset..])
			.char_indices()
			.skip(n)
			.map(|x| x.0)
			.next()
			.unwrap_or(text_len);
		self.skip(skip_len);
	}

	/// Return the span for the current line, excluding the EOL marker.
	pub fn cur_line(&mut self) -> Span<'a> {
		Span::new(self.src, self.pos, self.eol_pos(false))
	}

	/// Return the span for the whole text.
	fn text__(&mut self) -> Span<'a> {
		Span::new(self.src, Default::default(), self.eof_pos())
	}

	fn eof_pos(&mut self) -> Pos {
		if let Some(pos) = self.eof {
			pos
		} else {
			let mut pos = self.pos;
			pos.skip(&self.src[self.pos.offset..]);
			self.eof = Some(pos);
			pos
		}
	}

	fn eol_pos(&mut self, include_eol: bool) -> Pos {
		let (mut pos, eol) = loop {
			if let Some((pos, eol)) = self.eol {
				if pos.offset > self.pos.offset {
					break (pos, eol);
				}
			}

			let src = &self.src[self.pos.offset..];
			let (pos, eol) = if let Some(index) = src.find(|c| c == '\r' || c == '\n') {
				let mut new_pos = self.pos;
				new_pos.skip(&src[..index]);

				let bytes = src.as_bytes();
				if (bytes[index] as char) == '\r' && index < bytes.len() && (bytes[index + 1] as char) == '\n' {
					(new_pos, 2)
				} else {
					(new_pos, 1)
				}
			} else {
				(self.eof_pos(), 0)
			};

			break (pos, eol);
		};

		if include_eol && eol > 0 {
			pos.skip(&self.src[pos.offset..pos.offset + eol]);
		}

		pos
	}
}

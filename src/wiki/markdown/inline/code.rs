use regex::Regex;

use super::{Pos, Span, SpanIter};

pub struct ParsedInlineCode<'a> {
	/// The delimiter used for opening/closing.
	pub delim: &'a str,
	/// Span containing the code if the match was successful.
	pub code: Option<Span<'a>>,
	/// End position of the code.
	///
	/// This is the start position if the match was unsuccessful.
	///
	/// Even if the match was successful, this can be different
	/// than `self.code.end` because the first and last space of
	/// the code may be trimmed.
	pub end: Pos,
}

/// Parse an inline code starting at the current position.
pub fn parse<'a>(iter: &SpanIter<'a>) -> ParsedInlineCode<'a> {
	let start = iter.pos();
	let mut iter = iter.clone();
	let (delim, code) = parse_code_delim(&mut iter);
	let (code, end) = if let Some((mut span, spaced)) = code {
		let end = span.end;
		if spaced {
			let _ = span.start.skip_if(span.buffer, " ")
				|| span.start.skip_if(span.buffer, "\r\n")
				|| span.start.skip_if(span.buffer, "\n")
				|| span.start.skip_if(span.buffer, "\r");

			let buffer = span.buffer.as_bytes();
			let last_char = buffer[span.end.offset - 1] as char;
			match last_char {
				' ' => {
					span.end.offset -= 1;
					span.end.column -= 1;
				}
				'\r' | '\n' => {
					if last_char == '\n' && (buffer[span.end.offset - 2] as char) == '\r' {
						span.end.offset -= 2;
					} else {
						span.end.offset -= 1;
					}
					span.end.line -= 1;
					span.end.column = 9999;
				}
				_ => {}
			}
		}
		(Some(span), end)
	} else {
		(None, start)
	};

	ParsedInlineCode {
		delim: delim,
		code:  code,
		end:   end,
	}
}

fn parse_code_delim<'a>(iter: &mut SpanIter<'a>) -> (&'a str, Option<(Span<'a>, bool)>) {
	lazy_static! {
		static ref RE_DELIM_STA: Regex = Regex::new(r"^[`]+").unwrap();
		static ref RE_DELIM_END: Regex = Regex::new(r"[`]+").unwrap();
	}
	let delim = RE_DELIM_STA.find(iter.chunk()).unwrap().as_str();
	iter.skip_len(delim.len());

	let mut only_spaces = true;
	let sta = iter.pos();
	let end = iter.search_text(|s| {
		for m in RE_DELIM_END.find_iter(s) {
			if m.as_str().len() == delim.len() {
				let index = m.start();
				if s[..index].trim().len() > 0 {
					only_spaces = false;
				}
				return Some(index);
			}
		}
		if s.trim().len() > 0 {
			only_spaces = false;
		}
		None
	});

	if let Some(end) = end {
		let span = iter.span().sub_pos(sta..end);
		let text = span.text();
		let space_sta = !only_spaces
			&& match text.chars().next() {
				Some(' ') | Some('\r') | Some('\n') => true,
				_ => false,
			};
		let space_end = !only_spaces
			&& match text.chars().rev().next() {
				Some(' ') | Some('\r') | Some('\n') => true,
				_ => false,
			};
		(delim, Some((span, space_sta && space_end)))
	} else {
		(delim, None)
	}
}

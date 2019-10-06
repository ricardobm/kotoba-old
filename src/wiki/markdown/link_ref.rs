use super::block_parser::Leaf;
use super::{Span, SpanIter};

/// Parse a link reference definition.
///
/// Returns either a [Leaf::LinkReference] or a [Leaf::Paragraph].
///
/// The basic syntax for a reference definition is:
///
///     [label]: url 'title'
///
/// - Both `label` and `title` may span multiple lines.
/// - The `url` must be separated from the preceding `:` and the following
///   title by whitespace and at most one line break.
/// - All fields support escape sequences.
/// - The `url` can be:
///   - A bracketed `< >` sequence of zero or more characters, except for
///     line breaks and unescaped `<` or `>`.
///   - A non-empty sequence of characters, not including spaces or control
///     characters.
///   - For the non-bracketed syntax, parenthesis `()` are only allowed if
///     balanced or escaped.
/// - The `title` can be:
///   - A sequence of zero or more characters quoted by either `''` or `""`.
///   - A sequence of zero or more characters between matching parenthesis `()`.
///
///
/// NOTE
/// ----
/// Per the spec, no blank lines are allowed and the base indentation
/// should be at most 3 spaces. This function assumes that the [Span] is
/// coming from a [Paragraph] and as such those conditions are already met.
pub fn parse_link_ref<'a>(span: Span<'a>) -> Leaf<'a> {
	let mut iter = span.iter();
	iter.skip_spaces(false);
	if let Some(label) = parse_link_label(&mut iter) {
		if let Some(':') = iter.next_char() {
			iter.skip_char();
			iter.skip_spaces(true);
			if let Some(dest) = parse_link_destination(&mut iter) {
				iter.skip_spaces(true);
				let title = parse_link_title(&mut iter);

				iter.skip_spaces(true);
				if iter.at_end() {
					return Leaf::LinkReference {
						url:   dest,
						label: label,
						title: title,
					};
				}
			}
		}
	}

	Leaf::Paragraph { text: span }
}

fn parse_link_label<'a>(iter: &mut SpanIter<'a>) -> Option<Span<'a>> {
	// A link label begins with a left bracket (`[`) and ends with the
	// first right bracket (`]`) that is not backslash-escaped. Between
	// these brackets there must be at least one non-whitespace character.
	//
	// Unescaped square bracket characters are not allowed inside the opening
	// and closing square brackets of link labels. A link label can have at
	// most 999 characters inside the square brackets.
	if let Some('[') = iter.next_char() {
		iter.skip_char();
		let mut char_count = 0;
		let label_start = iter.pos();

		// Look for the end of the label.
		let mut is_empty = true;
		let label_end = loop {
			match iter.next_char() {
				Some('\\') => {
					// skip escape sequence
					is_empty = false;
					iter.skip_char();
					iter.skip_char();
					char_count += 2;
				}

				Some(']') => {
					let pos = iter.pos();
					iter.skip_char();
					break pos;
				}

				Some('[') => {
					return None;
				}

				Some(c) => {
					char_count += 1;
					iter.skip_char();
					if !c.is_ascii_whitespace() {
						is_empty = false;
					}
				}

				None => {
					return None;
				}
			}

			if char_count > 999 {
				return None;
			}
		};

		if is_empty {
			None
		} else {
			Some(iter.span().sub_pos(label_start..label_end))
		}
	} else {
		None
	}
}

fn parse_link_destination<'a>(iter: &mut SpanIter<'a>) -> Option<Span<'a>> {
	// A link destination consists of either
	let (start, end) = if let Some('<') = iter.next_char() {
		// - a sequence of zero or more characters between an opening `<` and a
		//   closing `>` that contains no line breaks or unescaped `<` or `>`
		//   characters, or
		iter.skip_char();
		let start = iter.pos();
		let end = loop {
			match iter.next_char() {
				Some('\\') => {
					// skip escape sequence
					iter.skip_char();
					iter.skip_char();
				}
				Some('<') => {
					return None;
				}
				Some('>') => {
					let pos = iter.pos();
					iter.skip_char();
					break pos;
				}
				Some(c) if c == '\n' || c == '\r' => {
					return None;
				}
				Some(_) => {
					iter.skip_char();
				}
				None => {
					return None;
				}
			}
		};
		(start, end)
	} else {
		// - a nonempty sequence of characters that does not start with `<`, does
		//   not include ASCII space or control characters, and includes parentheses
		//   only if (a) they are backslash-escaped or (b) they are part of a
		//   balanced pair of unescaped parentheses.
		let mut paren = 0;
		let start = iter.pos();
		let end = loop {
			match iter.next_char() {
				Some('\\') => {
					// skip escape sequence
					iter.skip_char();
					iter.skip_char();
				}
				Some('(') => {
					paren += 1;
					iter.skip_char();
				}
				Some(')') => {
					if paren == 0 {
						return None;
					}
					paren -= 1;
					iter.skip_char();
				}
				Some(c) if c.is_ascii_whitespace() || c.is_ascii_control() => {
					break iter.pos();
				}
				Some(_) => {
					iter.skip_char();
				}
				None => {
					break iter.pos();
				}
			}
		};
		if paren != 0 {
			return None;
		}
		(start, end)
	};
	if start == end {
		None
	} else {
		Some(iter.span().sub_pos(start..end))
	}
}

fn parse_link_title<'a>(iter: &mut SpanIter<'a>) -> Option<Span<'a>> {
	// A link title consists of either
	let (start, end) = if let Some('\'') | Some('"') = iter.next_char() {
		// - a sequence of zero or more characters between straight
		//   double-quote characters ("), including a `"` character
		//   only if it is backslash-escaped, or
		// - a sequence of zero or more characters between straight
		//   single-quote characters ('), including a `'` character
		//   only if it is backslash-escaped, or
		let delim = iter.next_char().unwrap();
		iter.skip_char();
		let start = iter.pos();
		let end = loop {
			match iter.next_char() {
				Some('\\') => {
					// skip escape sequence
					iter.skip_char();
					iter.skip_char();
				}
				Some(c) if c == delim => {
					let pos = iter.pos();
					iter.skip_char();
					break pos;
				}
				Some(_) => {
					iter.skip_char();
				}
				None => {
					return None;
				}
			}
		};
		(start, end)
	} else if let Some('(') = iter.next_char() {
		// - a sequence of zero or more characters between matching
		//   parentheses (`(...)`), including a `(` or `)` character
		//   only if it is backslash-escaped.
		let mut paren = 1;
		iter.skip_char();
		let start = iter.pos();
		let end = loop {
			match iter.next_char() {
				Some('\\') => {
					// skip escape sequence
					iter.skip_char();
					iter.skip_char();
				}
				Some('(') => {
					paren += 1;
					iter.skip_char();
				}
				Some(')') => {
					let pos = iter.pos();
					iter.skip_char();
					paren -= 1;
					if paren == 0 {
						break pos;
					}
				}
				Some(_) => {
					iter.skip_char();
				}
				None => {
					return None;
				}
			}
		};
		(start, end)
	} else {
		return None;
	};

	Some(iter.span().sub_pos(start..end))
}

use super::block_parser::Leaf;
use super::{Span, SpanIter};

/// Parse a link reference definition.
///
/// If `allow_partial` is true, this will parse partial link reference
/// definitions as valid.
///
/// Returns the parsed link reference definition and a boolean indicating
/// whether is it a complete definition. An incomplete definition may happen
/// if `allow_partial` is true, or for a valid link reference that has no
/// title.
///
/// Note that if `allow_partial` is true, the returned link reference fields
/// other than [span] may be garbage.
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
pub fn parse_link_ref<'a>(span: Span<'a>, allow_partial: bool) -> Option<(Leaf<'a>, bool)> {
	let mut iter = span.iter();
	let partial_ref = Some((
		Leaf::LinkReference {
			span:  span.clone(),
			url:   Span::default(),
			label: Span::default(),
			title: None,
		},
		false,
	));

	iter.skip_spaces(false);
	if let Some((label, closed)) = parse_link_label(&mut iter, allow_partial) {
		if !closed {
			debug_assert!(iter.at_end());
			return partial_ref;
		}

		if let Some(':') = iter.next_char() {
			iter.skip_char();
			iter.skip_spaces(true);
			if let Some((dest, closed)) = parse_link_destination(&mut iter, allow_partial) {
				if !closed {
					debug_assert!(iter.at_end());
					return partial_ref;
				}

				// link reference title must be separated by space
				if let Some(chr) = iter.next_char() {
					if !chr.is_whitespace() {
						return None;
					}
				}

				iter.skip_spaces(true);
				if let Some((title, closed)) = parse_link_title(&mut iter, allow_partial) {
					if !closed {
						debug_assert!(iter.at_end());
						return partial_ref;
					}

					iter.skip_spaces(true);
					if !iter.at_end() {
						return None;
					}

					let complete = title.is_some();
					return Some((
						Leaf::LinkReference {
							span:  span,
							url:   dest,
							label: label,
							title: title,
						},
						complete,
					));
				}
			}
		} else {
			// since we iterate chunks by line, we will never have a break
			// between the `]` and `:` in a link reference
			return None;
		}
	}

	None
}

fn parse_link_label<'a>(iter: &mut SpanIter<'a>, allow_partial: bool) -> Option<(Span<'a>, bool)> {
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
					return if allow_partial {
						Some((Span::default(), false))
					} else {
						None
					};
				}
			}

			if char_count > 999 {
				return None;
			}
		};

		if is_empty {
			None
		} else {
			Some((iter.span().sub_pos(label_start..label_end), true))
		}
	} else {
		None
	}
}

fn parse_link_destination<'a>(iter: &mut SpanIter<'a>, allow_partial: bool) -> Option<(Span<'a>, bool)> {
	// A link destination consists of either
	let (start, end, allow_empty) = if let Some('<') = iter.next_char() {
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
					return if allow_partial {
						Some((Span::default(), false))
					} else {
						None
					};
				}
			}
		};
		(start, end, true)
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
			return if allow_partial {
				Some((Span::default(), false))
			} else {
				None
			};
		}
		(start, end, false)
	};
	if start == end && !allow_empty {
		if allow_partial {
			Some((Span::default(), false))
		} else {
			None
		}
	} else {
		Some((iter.span().sub_pos(start..end), true))
	}
}

fn parse_link_title<'a>(iter: &mut SpanIter<'a>, allow_partial: bool) -> Option<(Option<Span<'a>>, bool)> {
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
					return if allow_partial { Some((None, false)) } else { None };
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
					return if allow_partial { Some((None, false)) } else { None };
				}
			}
		};
		(start, end)
	} else if iter.at_end() {
		return Some((None, true));
	} else {
		return None;
	};

	Some((Some(iter.span().sub_pos(start..end)), true))
}

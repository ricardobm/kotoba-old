use super::inline_code;
use super::Elem;
use super::LinkReference;
use super::LinkReferenceMap;
use super::Pos;
use super::PosRange;
use super::Span;
use super::SpanIter;
use super::TextMode;
use super::TextNode;

#[derive(Clone, Debug)]
pub struct Link<'a> {
	pub title:    Option<TextNode<'a>>,
	pub url:      Option<TextNode<'a>>,
	pub children: Vec<Elem<'a>>,
	pub range:    PosRange,
}

pub fn parse<'a>(iter: &SpanIter<'a>, refs: &LinkReferenceMap<'a>) -> Option<Link<'a>> {
	let mut iter = iter.clone();
	let start = iter.pos();
	let label = if let Some(label) = parse_link_label(&mut iter, refs) {
		label
	} else {
		return None;
	};

	if let Some((url, title, end)) = parse_inline_link(&iter) {
		let children = super::parse_inline(&label, refs);
		Some(Link {
			title:    title,
			url:      url,
			children: children,
			range:    PosRange { start, end },
		})
	} else if let Some(ref_label) = parse_link_label(&mut iter, refs) {
		let end = iter.pos();
		if let Some(link_ref) = refs.get(&ref_label) {
			Some(reference_to_link(link_ref, label, refs, start, end))
		} else {
			None
		}
	} else {
		// handle a shortcut link reference
		if iter.chunk().starts_with("[]") {
			// skip empty label
			iter.skip_bytes(2);
		}

		let end = iter.pos();
		if let Some(link_ref) = refs.get(&label) {
			Some(reference_to_link(link_ref, label, refs, start, end))
		} else {
			None
		}
	}
}

fn reference_to_link<'a>(
	reference: &LinkReference<'a>,
	label: Span<'a>,
	refs: &LinkReferenceMap<'a>,
	start: Pos,
	end: Pos,
) -> Link<'a> {
	let children = super::parse_inline(&label, refs);
	Link {
		title:    Some(TextNode::new(reference.title.clone(), TextMode::WithEscapesAndEntities)),
		url:      Some(TextNode::new(reference.url.clone(), TextMode::WithEscapesAndEntities)),
		children: children,
		range:    PosRange { start, end },
	}
}

fn parse_link_label<'a>(iter: &mut SpanIter<'a>, refs: &LinkReferenceMap<'a>) -> Option<Span<'a>> {
	if let Some('[') = iter.next_char() {
		let mut bracket_count = 1;
		iter.skip_char();
		let label_start = iter.pos();

		// Look for the end of the label.
		let mut is_empty = true;
		let label_end = loop {
			if let Some(pos) = iter.search_char(is_special_char) {
				if is_empty {
					is_empty = iter.span().sub_pos(iter.pos()..pos).text().trim().len() == 0;
				}
				iter.skip_to(pos);
			}

			match iter.next_char() {
				Some('[') => {
					// links cannot contain other links
					if let Some(_) = parse(&iter, refs) {
						return None;
					}
					// links may contain matched pairs of brackets
					bracket_count += 1;
					iter.skip_char();
				}

				Some(']') => {
					let end = iter.pos();
					bracket_count -= 1;
					iter.skip_char();
					if bracket_count == 0 {
						break end;
					}
				}

				Some('\\') => {
					// skip escape sequence
					is_empty = false;
					iter.skip_char();
					iter.skip_char();
				}

				Some('`') => {
					// backtick code spans bind more tightly than links
					is_empty = false;
					let (code, delim) = inline_code::parse(&iter);
					if let Some(code) = code {
						iter.skip_to(code.range.end);
					} else {
						iter.skip_bytes(delim.len());
					}
				}

				Some('<') => {
					is_empty = false;
					// raw HTML and autolinks also bind more tightly than links
					if let None = super::parse_left_angle_bracket(iter) {
						iter.skip_char();
					}
				}

				Some(c) => {
					if !c.is_ascii_whitespace() {
						is_empty = false;
					}
					iter.skip_char();
				}

				// End of paragraph mid label
				None => return None,
			}
		};

		if !is_empty {
			let label = iter.span().sub_pos(label_start..label_end);
			Some(label)
		} else {
			None
		}
	} else {
		None
	}
}

fn parse_inline_link<'a>(iter: &SpanIter<'a>) -> Option<(Option<TextNode<'a>>, Option<TextNode<'a>>, Pos)> {
	let mut iter = iter.clone();
	if let Some('(') = iter.next_char() {
		iter.skip_char();
		iter.skip_spaces(true);
		let (dest, mut iter) = {
			let mut new_iter = iter.clone();
			let dest = parse_link_destination(&mut new_iter);
			if let Some(dest) = dest {
				(Some(dest), new_iter)
			} else {
				(None, iter)
			}
		};

		iter.skip_spaces(true);
		let (title, mut iter) = {
			let mut new_iter = iter.clone();
			let title = parse_link_title(&mut new_iter);
			if let Some(title) = title {
				(Some(title), new_iter)
			} else {
				(None, iter)
			}
		};

		iter.skip_spaces(true);
		if let Some(')') = iter.next_char() {
			iter.skip_char();
			Some((dest, title, iter.pos()))
		} else {
			None
		}
	} else {
		None
	}
}

fn parse_link_destination<'a>(iter: &mut SpanIter<'a>) -> Option<TextNode<'a>> {
	// A link destination consists of either...
	if let Some('<') = iter.next_char() {
		// ...a sequence of zero or more characters between an opening `<` and
		// a closing `>` that contains no line breaks or unescaped `<` or `>`
		// characters

		iter.skip_char();
		let start = iter.pos();
		let end = loop {
			match iter.next_char() {
				Some('>') => {
					let end = iter.pos();
					iter.skip_char();
					break end;
				}

				Some('\\') => {
					// skip escape sequence.
					iter.skip_char();
					iter.skip_char();
				}

				None | Some('\n') | Some('\r') => return None,

				Some(_) => {
					iter.skip_char();
				}
			}
		};

		Some(TextNode::new(
			iter.span().sub_pos(start..end),
			TextMode::WithEscapesAndEntities,
		))
	} else {
		// ...a nonempty sequence of characters that does not start with `<`,
		// does not include ASCII space or control characters, and includes
		// parentheses only if (a) they are backslash-escaped or (b) they are
		// part of a balanced pair of unescaped parentheses

		let mut paren_count = 0;
		let start = iter.pos();
		let end = loop {
			match iter.next_char() {
				Some('\\') => {
					// skip escape sequence.
					iter.skip_char();
					iter.skip_char();
				}

				Some('(') => {
					iter.skip_char();
					paren_count += 1;
				}

				Some(')') => {
					if paren_count == 0 {
						break iter.pos();
					}
					paren_count -= 1;
					iter.skip_char();
				}

				Some(c) if c.is_ascii_whitespace() || c.is_ascii_control() => break iter.pos(),

				Some(_) => {
					iter.skip_char();
				}

				None => break iter.pos(),
			}
		};

		if end == start {
			None
		} else {
			Some(TextNode::new(
				iter.span().sub_pos(start..end),
				TextMode::WithEscapesAndEntities,
			))
		}
	}
}

fn parse_link_title<'a>(iter: &mut SpanIter<'a>) -> Option<TextNode<'a>> {
	// A link title consists of either...
	let (start, end) = match iter.next_char() {
		// - a sequence of zero or more characters between straight
		//   double-quote characters ("), including a `"` character
		//   only if it is backslash-escaped, or
		// - a sequence of zero or more characters between straight
		//   single-quote characters ('), including a ' character only
		//   if it is backslash-escaped, or
		Some(quote) if quote == '\'' || quote == '"' => {
			iter.skip_char();
			let start = iter.pos();
			let end = loop {
				match iter.next_char() {
					Some('\\') => {
						// consume escape sequence
						iter.skip_char();
						iter.skip_char();
					}

					Some(c) if c == quote => {
						let end = iter.pos();
						iter.skip_char();
						break end;
					}

					Some(_) => {
						iter.skip_char();
					}

					None => return None,
				}
			};

			(start, end)
		}

		// - a sequence of zero or more characters between matching
		//   parentheses (`(...)`), including a `(` or `)` character
		//   only if it is backslash-escaped
		Some('(') => {
			iter.skip_char();
			let start = iter.pos();
			let end = loop {
				match iter.next_char() {
					Some('\\') => {
						// consume escape sequence
						iter.skip_char();
						iter.skip_char();
					}

					Some(')') => {
						let end = iter.pos();
						iter.skip_char();
						break end;
					}

					None | Some('(') => return None,

					Some(_) => {
						iter.skip_char();
					}
				}
			};

			(start, end)
		}

		_ => return None,
	};

	Some(TextNode::new(
		iter.span().sub_pos(start..end),
		TextMode::WithEscapesAndEntities,
	))
}

fn is_special_char(c: char) -> bool {
	match c {
		'[' | ']' | '<' | '`' | '\\' => true,
		_ => false,
	}
}

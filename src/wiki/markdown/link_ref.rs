use super::block_parser::Leaf;
use super::common;
use super::{Range, Span, SpanIter};

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
	let iter = span.iter();
	let (iter, label) = parse_link_label(&span, iter, "", true);
	if let Some((label, text)) = label {
		let (iter, url) = parse_link_url(&span, iter, text, true);
		if let Some((url, text)) = url {
			let (iter, title) = parse_link_title(&span, iter, text, true);
			if let Some((title, text)) = title {
				let mut iter = iter;
				let empty = text.trim().len() == 0 && iter.all(|x| x.trim().len() == 0);
				if empty {
					let buffer = span.buffer;
					return Leaf::LinkReference {
						url:   &buffer[url.start..url.end],
						label: span.sub(label),
						title: span.sub(title),
					};
				}
			}
		}
	}

	Leaf::Paragraph { text: span }
}

fn parse_link_label<'a>(
	span: &Span<'a>,
	mut iter: SpanIter<'a>,
	mut text: &'a str,
	as_ref: bool,
) -> (SpanIter<'a>, Option<(Range, &'a str)>) {
	enum S<'a> {
		Start,
		LabelSta,
		LabelMid(usize),
		LabelEnd(usize, usize, usize),
		End(usize, usize, &'a str),
		None,
	}

	let mut state = S::Start;

	'main: loop {
		text = text.trim();
		while text.len() == 0 {
			text = if let Some(text) = iter.next() {
				text.trim_start()
			} else {
				break;
			}
		}

		loop {
			// Inner loop invariants:
			// - `text` is never empty
			// - `text` does not start at a space
			// - `text` is always within range
			let range = span.text_range(text).unwrap();

			state = match state {
				S::Start => {
					// Look for the label start `[`
					if !text.starts_with('[') {
						S::None
					} else {
						text = &text[1..];
						S::LabelSta
					}
				}

				S::LabelSta => {
					// Setup the label start
					S::LabelMid(range.start)
				}

				S::LabelMid(start) => {
					// Consume the label content
					if let Some(index) = text.find('\\') {
						// Skip over escaped character
						text = common::skip_chars(&text[index..], 2);
						S::LabelMid(start)
					} else if let Some(index) = text.find(if as_ref { "]:" } else { "]" }) {
						let delim_len = if as_ref { 2 } else { 1 };
						// Found the final `]:`
						text = &text[index + delim_len..];
						let end = range.start + index;
						S::LabelEnd(start, end, end + delim_len)
					} else {
						// Current span has no `]:`
						continue 'main;
					}
				}

				S::LabelEnd(start, end, delim_end) => {
					// Any text here is the start of the URL
					if as_ref && range.start == delim_end {
						// There should be at least some space between
						// the ':' and the next part
						S::None
					} else {
						S::End(start, end, text)
					}
				}

				S::None | S::End(..) => {
					break 'main;
				}
			};

			// Maintain inner loop invariants
			text = text.trim();
			if text.len() == 0 {
				continue 'main;
			}
		}
	}

	if let S::End(start, end, text) = state {
		(iter, Some((Range { start, end }, text)))
	} else {
		(iter, None)
	}
}

fn parse_link_url<'a>(
	span: &Span<'a>,
	mut iter: SpanIter<'a>,
	mut text: &'a str,
	as_ref: bool,
) -> (SpanIter<'a>, Option<(Range, &'a str)>) {
	enum S<'a> {
		Start,
		Bracket(usize),
		Text(usize, usize),
		BeforeEnd(usize, usize, usize),
		End(usize, usize, &'a str),
		None,
	}

	let mut state = S::Start;

	'main: loop {
		text = text.trim();
		while text.len() == 0 {
			text = if let Some(text) = iter.next() {
				text.trim_start()
			} else {
				break;
			}
		}

		loop {
			// Inner loop invariants:
			// - `text` is never empty
			// - `text` does not start at a space
			// - `text` is always within range
			let range = span.text_range(text).unwrap();

			state = match state {
				S::Start => {
					if text.starts_with('<') {
						text = &text[1..];
						S::Bracket(range.start + 1)
					} else {
						S::Text(range.start, 0)
					}
				}

				S::Bracket(start) => {
					if let Some(index) = text.find('\\') {
						// skip escaped character
						text = common::skip_chars(&text[index..], 2);
						S::Bracket(start)
					} else if let Some(index) = text.find('>') {
						// found the delimiter
						text = &text[index + 1..];
						S::BeforeEnd(start, index, index + 1)
					} else if let Some(_) = text.find('\n').or_else(|| text.find('\r')) {
						// the `< >` syntax does not allow line breaks
						S::None
					} else {
						text = "";
						S::Bracket(start)
					}
				}

				S::Text(start, level) => {
					let mut valid = true;
					let mut index = 0;
					let mut level = level;
					let mut escaped = false;
					let mut closed = false;
					for (idx, chr) in text.char_indices() {
						if escaped {
							escaped = false;
						} else if chr.is_control() {
							closed = true;
							valid = false;
							break;
						} else if chr.is_whitespace() {
							index = idx;
							break;
						} else if chr == '(' {
							level += 1;
						} else if chr == ')' {
							if level == 0 {
								valid = false;
								break;
							} else {
								level -= 1;
							}
						} else if chr == '\\' {
							escaped = true;
						}
					}

					if !valid {
						S::None
					} else if closed {
						if level != 0 {
							S::None
						} else {
							text = &text[index..];
							S::BeforeEnd(start, index, index)
						}
					} else {
						text = "";
						S::Text(start, level)
					}
				}

				S::BeforeEnd(start, end, delim_end) => {
					if as_ref && range.start == delim_end {
						// There should be at least some space between
						// the delimiter and this.
						S::None
					} else {
						S::End(start, end, text)
					}
				}

				S::None | S::End(..) => {
					break 'main;
				}
			};

			// Maintain inner loop invariants
			text = text.trim();
			if text.len() == 0 {
				continue 'main;
			}
		}
	}

	if let S::End(start, end, text) = state {
		(iter, Some((Range { start, end }, text)))
	} else {
		(iter, None)
	}
}

fn parse_link_title<'a>(
	span: &Span<'a>,
	mut iter: SpanIter<'a>,
	mut text: &'a str,
	_as_ref: bool,
) -> (SpanIter<'a>, Option<(Range, &'a str)>) {
	enum S<'a> {
		Start,
		Delim(char, usize),
		Paren(u32, usize),
		End(usize, usize, &'a str),
		None,
	}

	let mut state = S::Start;

	'main: loop {
		text = text.trim();
		while text.len() == 0 {
			text = if let Some(text) = iter.next() {
				text.trim_start()
			} else {
				break;
			}
		}

		loop {
			// Inner loop invariants:
			// - `text` is never empty
			// - `text` does not start at a space
			// - `text` is always within range
			let range = span.text_range(text).unwrap();

			state = match state {
				S::Start => {
					let delim = text.chars().next().unwrap();
					text = &text[1..];
					if delim == '\'' || delim == '"' {
						S::Delim(delim, range.start + 1)
					} else if delim == '(' {
						S::Paren(1, range.start + 1)
					} else {
						S::None
					}
				}

				S::Delim(delim, start) => {
					if let Some(index) = text.find('\\') {
						// Skip escape sequence
						text = common::skip_chars(&text[index..], 2);
						S::Delim(delim, start)
					} else if let Some(index) = text.find(delim) {
						text = common::skip_chars(&text[index..], 1);
						S::End(start, range.start + index + 1, text)
					} else {
						text = "";
						S::Delim(delim, start)
					}
				}

				S::Paren(count, start) => {
					if count == 0 {
						S::None
					} else {
						let mut count = count;
						let mut escaped = false;
						let mut valid = true;
						let mut end = 0;
						for (idx, chr) in text.char_indices() {
							if escaped {
								escaped = false;
							} else if chr == '(' {
								count += 1;
							} else if chr == ')' {
								if count == 0 {
									valid = false;
									break;
								} else if count == 1 {
									count = 0;
									text = &text[idx + 1..];
									valid = text.trim().len() == 0;
									end = idx;
									break;
								} else {
									count -= 1;
								}
							} else if chr == '\\' {
								escaped = true;
							}
						}

						if !valid {
							S::None
						} else if count == 0 {
							S::End(start, end, text)
						} else {
							S::Paren(count, start)
						}
					}
				}

				S::None | S::End(..) => {
					break 'main;
				}
			};

			// Maintain inner loop invariants
			text = text.trim();
			if text.len() == 0 {
				continue 'main;
			}
		}
	}

	if let S::End(start, end, text) = state {
		(iter, Some((Range { start, end }, text)))
	} else {
		(iter, None)
	}
}

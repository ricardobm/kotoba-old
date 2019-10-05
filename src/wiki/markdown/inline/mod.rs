use std::collections::VecDeque;

use super::html::html_entity;
use super::{LinkReference, LinkReferenceMap};
use super::{Pos, PosRange, Range, Span, SpanIter};

mod entities;

mod autolink;
mod emphasis;
mod inline_code;
mod inline_text;
mod link;
mod raw_html;

pub use self::autolink::AutoLink;
pub use self::inline_code::CodeNode;
pub use self::inline_text::{TextMode, TextNode, TextOrChar, TextSpan};
pub use self::link::Link;

dbg_flag!(false);

/// An inline element.
#[derive(Clone, Debug)]
pub enum Elem<'a> {
	// An inline element tag.
	Tag(InlineTag, Vec<Elem<'a>>),
	/// Text content to output.
	Text(TextNode<'a>),
	/// Inline code element.
	Code(CodeNode<'a>),
	/// A `< >` delimited autolink.
	AutoLink(AutoLink<'a>),
	/// Inline link.
	Link(Link<'a>),
	/// Raw HTML to generate verbatim.
	HTML(Span<'a>),
}

#[derive(Clone, Debug)]
pub enum InlineTag {
	Emphasis,
	Strong,
	Strikethrough,
}

impl InlineTag {
	pub fn html_tag(&self) -> &'static str {
		match self {
			InlineTag::Emphasis => "em",
			InlineTag::Strong => "strong",
			InlineTag::Strikethrough => "del",
		}
	}
}

use self::emphasis::Delim;

pub fn parse_inline<'a>(span: &Span<'a>, refs: &LinkReferenceMap<'a>) -> Vec<Elem<'a>> {
	let mut iter = span.iter();
	let mut helper = ParserHelper::new(iter.pos());
	while !iter.at_end() {
		// search for the next possible non-text element on the chunk
		if let Some(start) = iter.find_char_in_chunk(is_syntax_char) {
			iter.skip_to(start);
			helper.push_text(start);

			// Try to parse the element. This will either return the parsed
			// element skipping to the end of it, or skip the unmatched
			// delimiter and return None.
			let start = iter.pos();
			match iter.next_char() {
				Some('`') => {
					let (code, delim) = inline_code::parse(&iter);
					if let Some(code) = code {
						iter.skip_to(code.range.end);
						helper.push_elem(start, iter.pos(), Elem::Code(code));
					} else {
						iter.skip_bytes(delim.len());
					}
				}
				Some('<') => {
					if let Some(elem) = parse_left_angle_bracket(&mut iter) {
						helper.push_elem(start, iter.pos(), elem);
					}
				}
				Some('[') => {
					if let Some(link) = link::parse(&iter, refs) {
						iter.skip_to(link.range.end);
						helper.push_elem(start, iter.pos(), Elem::Link(link));
					} else {
						iter.skip_char();
					}
				}
				Some('*') | Some('_') => {
					if let Some(delim) = emphasis::parse_delim(&mut iter) {
						helper.push_delim(start, iter.pos(), delim);
					} else {
						iter.skip_char();
					}
				}
				_ => {
					iter.skip_char();
				}
			};
		} else {
			// skip chunk and continue parsing
			iter.skip_chunk();
		}
	}

	// generate any suffix left as text
	helper.push_text(iter.pos());
	helper.to_result(&span)
}

fn parse_left_angle_bracket<'a>(iter: &mut SpanIter<'a>) -> Option<Elem<'a>> {
	debug_assert!(if let Some('<') = iter.next_char() { true } else { false });
	if let Some(link) = autolink::parse(iter) {
		Some(Elem::AutoLink(link))
	} else if let Some(span) = raw_html::parse(iter) {
		Some(Elem::HTML(span))
	} else {
		iter.skip_char();
		None
	}
}

fn is_syntax_char(c: char) -> bool {
	match c {
		// HTML or autolink
		'<' => true,
		// code spans
		'`' => true,
		// emphasis and strikethrough
		'*' | '_' | '~' => true,
		// links
		'[' | '!' => true,
		_ => false,
	}
}

//=============================================
// Emphasis parsing helper
//=============================================

/// Temporary parsed nodes while kept inside a [Container].
#[derive(Debug, Clone)]
enum Parsed<'a> {
	Text(Pos, Pos),
	Node(Elem<'a>),
	Tag(InlineTag, VecDeque<Parsed<'a>>),
}

/// Mantains an open emphasis/strong container and its children while parsing.
#[derive(Debug)]
struct Container<'a> {
	delim:     Delim<'a>,
	delim_pos: Pos,
	children:  VecDeque<Parsed<'a>>,
}

impl<'a> Container<'a> {
	fn can_be_closed_by(&self, delim: &Delim<'a>) -> bool {
		if self.delim.token.len() > 0 && delim.can_close && delim.token.starts_with(self.delim.token) {
			if delim.can_open || self.delim.can_close {
				// if one of the delimiters can both open and close emphasis,
				// then the sum of the lengths of the delimiter runs containing
				// the opening and closing delimiters must not be a multiple
				// of 3 unless both lengths are multiples of 3
				let d1 = &self.delim;
				let d2 = delim;
				(d1.length + d2.length) % 3 != 0 || (d1.length % 3 == 0 && d2.length % 3 == 0)
			} else {
				true
			}
		} else {
			false
		}
	}
}

/// Helper to manage the emphasis logic during parsing.
struct ParserHelper<'a> {
	parents: VecDeque<Container<'a>>,
	cursor:  Pos,
}

impl<'a> ParserHelper<'a> {
	fn new(cursor: Pos) -> ParserHelper<'a> {
		let mut out = ParserHelper {
			parents: Default::default(),
			cursor:  cursor,
		};
		out.parents.push_back(Container {
			delim:     Default::default(),
			delim_pos: cursor,
			children:  Default::default(),
		});
		out
	}

	fn parent(&mut self) -> &mut Container<'a> {
		self.parents.back_mut().unwrap()
	}

	fn push_text(&mut self, pos: Pos) {
		if pos > self.cursor {
			let cursor = self.cursor;
			self.parent().children.push_back(Parsed::Text(cursor, pos));
			self.cursor = pos;
		}
	}

	fn push_elem(&mut self, start: Pos, end: Pos, elem: Elem<'a>) {
		self.push_text(start);
		self.parent().children.push_back(Parsed::Node(elem));
		self.cursor = end;
	}

	fn push_delim(&mut self, start: Pos, end: Pos, mut delim: Delim<'a>) {
		self.push_text(start);

		let mut delim_pos = start;

		// close any parent emphasis containers that can be closed by the
		// delimiter.
		while delim.token.len() > 0 && self.parent().can_be_closed_by(&delim) {
			// pop the parent container, turn it into a tag and append to the
			// previous parent
			let p = self.parents.pop_back().unwrap();
			let tag = if p.delim.token.len() == 1 {
				InlineTag::Emphasis
			} else {
				InlineTag::Strong
			};
			let tag = Parsed::Tag(tag, p.children);
			self.parent().children.push_back(tag);
			// consume the used token from the delimiter
			let skip_len = p.delim.token.len();
			delim_pos.skip(&delim.token[..skip_len]);
			delim.token = &delim.token[skip_len..];
		}

		while delim.token.len() > 0 && delim.can_open {
			let token = if delim.token.len() == 2 {
				// an interpretation `<strong>...</strong>` is always preferred
				// to `<em><em>...</em></em>`
				delim.token
			} else {
				// an interpretation `<em><strong>...</strong></em>` is always
				// preferred to `<strong><em>...</em></strong>`
				&delim.token[..1]
			};
			delim.token = &delim.token[token.len()..];
			let mut open_delim = delim.clone();
			open_delim.token = token;
			self.parents.push_back(Container {
				delim:     open_delim,
				delim_pos: delim_pos,
				children:  Default::default(),
			});
			delim_pos.skip(token);
		}

		self.cursor = delim_pos;

		// push any remaining delimiter as plain text and move the cursor to
		// the new end
		if end > delim_pos {
			self.push_text(end);
		} else {
			debug_assert!(delim.token == "");
		}
	}

	fn to_result(mut self, span: &Span<'a>) -> Vec<Elem<'a>> {
		// Pop all unclosed container elements, generating their open delimiters
		// as plain text.
		while self.parents.len() > 1 {
			let mut old_par = self.parents.pop_back().unwrap();
			let cur_par = self.parent();
			let txt_sta = old_par.delim_pos;
			let txt_end = {
				let mut aux_pos = old_par.delim_pos;
				aux_pos.skip(old_par.delim.token);
				aux_pos
			};
			cur_par.children.push_back(Parsed::Text(txt_sta, txt_end));
			cur_par.children.append(&mut old_par.children);
		}

		let p = self.parents.pop_back().unwrap();
		Self::to_elements(p.children, span)
	}

	fn to_elements(mut parsed: VecDeque<Parsed<'a>>, span: &Span<'a>) -> Vec<Elem<'a>> {
		let mut out = Vec::new();

		let push_text = |ls: &mut Vec<Elem<'a>>, sta: Pos, end: Pos| {
			let text = TextNode::new(span.sub_pos(sta..end), TextMode::WithLinks);
			ls.push(Elem::Text(text));
		};

		let mut last_text = None;
		while let Some(elem) = parsed.pop_front() {
			match elem {
				Parsed::Node(elem) => {
					if let Some((sta, end)) = last_text {
						push_text(&mut out, sta, end);
						last_text = None;
					}
					out.push(elem);
				}
				Parsed::Tag(tag, children) => {
					if let Some((sta, end)) = last_text {
						push_text(&mut out, sta, end);
						last_text = None;
					}
					let children = Self::to_elements(children, span);
					out.push(Elem::Tag(tag, children));
				}
				Parsed::Text(new_sta, new_end) => {
					if let Some((cur_sta, cur_end)) = last_text {
						if cur_end == new_sta {
							last_text = Some((cur_sta, new_end));
						} else {
							push_text(&mut out, cur_sta, cur_end);
							last_text = Some((new_sta, new_end));
						}
					} else {
						last_text = Some((new_sta, new_end));
					}
				}
			}
		}

		if let Some((sta, end)) = last_text {
			push_text(&mut out, sta, end);
		}

		out
	}
}

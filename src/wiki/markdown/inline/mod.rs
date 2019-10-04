use super::html::html_entity;
use super::LinkReferenceMap;
use super::{Pos, PosRange, Range, Span, SpanIter};

mod entities;

mod inline_code;
pub use self::inline_code::CodeNode;
mod autolink;
pub use self::autolink::AutoLink;
mod inline_text;
pub use self::inline_text::{TextMode, TextNode, TextOrChar, TextSpan};

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

pub fn parse_inline<'a>(span: &Span<'a>, _refs: &LinkReferenceMap) -> Vec<Elem<'a>> {
	let mut iter = span.iter();
	let mut list = Vec::new();

	let push_text = |ls: &mut Vec<Elem<'a>>, sta: Pos, end: Pos| {
		let text = TextNode::new(span.sub_pos(sta..end), TextMode::WithLinks);
		let text = Elem::Text(text);
		dbg_val!(&text);
		ls.push(text);
	};

	let mut text_start = iter.pos();
	while !iter.at_end() {
		// search for the next possible non-text element on the chunk
		if let Some(start) = iter.find_char_in_chunk(is_syntax_char) {
			iter.skip_to(start);

			// Try to parse the element. This will either return the parsed
			// element skipping to the end of it, or skip the unmatched
			// delimiter and return None.
			let matched = match iter.next_char() {
				Some('`') => {
					let (code, delim) = inline_code::parse(&iter);
					if let Some(code) = code {
						iter.skip_to(code.range.end);
						Some(Elem::Code(code))
					} else {
						iter.skip_bytes(delim.len());
						None
					}
				}
				Some('<') => {
					if let Some(link) = autolink::parse(&mut iter) {
						Some(Elem::AutoLink(link))
					} else {
						iter.skip_char();
						None
					}
				}
				_ => {
					iter.skip_char();
					None
				}
			};

			if let Some(elem) = matched {
				// generate prefix as text
				if start > text_start {
					push_text(&mut list, text_start, start);
				}
				text_start = iter.pos();

				dbg_val!(&elem);

				// generate element
				list.push(elem);
			}
		} else {
			// skip chunk and continue parsing
			iter.skip_chunk();
		}
	}

	// generate any suffix left as text
	if iter.pos() > text_start {
		push_text(&mut list, text_start, iter.pos());
	}

	dbg_print!("parsed {} inline elements", list.len());

	list
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

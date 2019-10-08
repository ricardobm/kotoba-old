use std::fmt;
use std::fmt::Write;

use super::inline::{parse_inline, CodeNode, Elem, TextNode, TextOrChar, TextSpan};
use super::{Block, HeaderLevel, LinkReferenceMap, MarkupEvent};

pub fn output<'a>(f: &mut fmt::Formatter, event: &MarkupEvent<'a>, refs: &LinkReferenceMap<'a>) -> fmt::Result {
	match event {
		MarkupEvent::Inline(span) => {
			for elem in parse_inline(span, refs, false) {
				fmt_inline(f, elem)?;
			}
			Ok(())
		}
		MarkupEvent::InlineCell(span) => {
			for elem in parse_inline(span, refs, true) {
				fmt_inline(f, elem)?;
			}
			Ok(())
		}
		MarkupEvent::Raw(span) => {
			for html in span.iter() {
				f.write_str(html)?;
			}
			Ok(())
		}
		MarkupEvent::Code(span) => {
			for txt in span.iter() {
				for ch in txt.chars() {
					write_html_char(f, ch)?;
				}
			}
			Ok(())
		}
		MarkupEvent::Open(block) => fmt_block_tags(block, true, f),
		MarkupEvent::Close(block) => fmt_block_tags(block, false, f),
	}
}

#[inline(always)]
pub fn html_entity(c: char) -> Option<&'static str> {
	match c {
		'"' => Some("&quot;"),
		'&' => Some("&amp;"),
		'<' => Some("&lt;"),
		'>' => Some("&gt;"),
		'\'' => Some("&apos;"),
		'\0' => Some("\u{FFFD}"), // mandated by the spec
		_ => None,
	}
}

#[inline(always)]
fn write_html_char(f: &mut fmt::Formatter, c: char) -> fmt::Result {
	if let Some(entity) = html_entity(c) {
		f.write_str(entity)
	} else {
		f.write_char(c)
	}
}

fn escape_html(f: &mut fmt::Formatter, s: &str) -> fmt::Result {
	for c in s.chars() {
		write_html_char(f, c)?;
	}
	Ok(())
}

fn fmt_inline<'a>(f: &mut fmt::Formatter, elem: Elem<'a>) -> fmt::Result {
	match elem {
		Elem::Tag(tag, children) => {
			write!(f, "<{}>", tag.html_tag())?;
			for it in children {
				fmt_inline(f, it)?;
			}
			write!(f, "</{}>", tag.html_tag())?;
		}

		Elem::Code(CodeNode { text, .. }) => {
			f.write_str("<code>")?;
			fmt_text(f, text)?;
			f.write_str("</code>")?;
		}

		Elem::Text(text) => {
			fmt_text(f, text)?;
		}

		Elem::AutoLink(a) => {
			f.write_str("<a href=\"")?;
			if a.prefix.len() > 0 {
				f.write_str(a.prefix)?;
			}
			escape_html(f, a.link)?;
			f.write_str("\">")?;
			escape_html(f, a.link)?;
			f.write_str("</a>")?;
		}

		Elem::Link(a) => {
			f.write_str("<a href=\"")?;
			if let Some(url) = a.url {
				fmt_text(f, url)?;
			}

			f.write_str("\"")?;
			if let Some(title) = a.title {
				f.write_str(" title=\"")?;
				fmt_text(f, title)?;
				f.write_str("\"")?;
			}
			f.write_str(">")?;

			for it in a.children {
				fmt_inline(f, it)?;
			}

			f.write_str("</a>")?;
		}

		Elem::Image(img) => {
			f.write_str("<img src=\"")?;
			if let Some(url) = img.url {
				fmt_text(f, url)?;
			}

			f.write_str("\" alt=\"")?;
			for it in img.alt {
				fmt_inline_text(f, it)?;
			}
			f.write_str("\"")?;

			if let Some(title) = img.title {
				f.write_str(" title=\"")?;
				fmt_text(f, title)?;
				f.write_str("\"")?;
			}

			f.write_str("/>")?;
		}

		Elem::HTML(text) => {
			fmt_text(f, text)?;
		}
	}
	Ok(())
}

/// Same as [fmt_inline] but outputs only textual content.
fn fmt_inline_text<'a>(f: &mut fmt::Formatter, elem: Elem<'a>) -> fmt::Result {
	match elem {
		Elem::Tag(_tag, children) => {
			for it in children {
				fmt_inline_text(f, it)?;
			}
		}

		Elem::Code(CodeNode { text, .. }) => {
			fmt_text(f, text)?;
		}

		Elem::Text(text) => {
			fmt_text_with_mode(f, text, Mode::TextOnly)?;
		}

		Elem::AutoLink(a) => {
			escape_html(f, a.link)?;
		}

		Elem::Link(a) => {
			for it in a.children {
				fmt_inline_text(f, it)?;
			}
		}

		Elem::Image(img) => {
			for it in img.alt {
				fmt_inline_text(f, it)?;
			}
		}

		Elem::HTML(_span) => {}
	}
	Ok(())
}

enum Mode {
	Normal,
	TextOnly,
}

fn fmt_text<'a>(f: &mut fmt::Formatter, node: TextNode<'a>) -> fmt::Result {
	fmt_text_with_mode(f, node, Mode::Normal)
}

fn fmt_text_with_mode<'a>(f: &mut fmt::Formatter, node: TextNode<'a>, mode: Mode) -> fmt::Result {
	let text_only = if let Mode::TextOnly = mode { true } else { false };
	for text in node.iter() {
		match text {
			TextSpan::Text(s) => f.write_str(s)?,
			TextSpan::Char(c) => f.write_char(c)?,
			TextSpan::LineBreak => {
				if text_only {
					f.write_str(" ")?;
				} else {
					f.write_str("<br/>\n")?;
				}
			}
			TextSpan::Entity { entity, output, .. } => {
				if entity == "&nbsp;" {
					f.write_str(entity)?;
				} else {
					match output {
						TextOrChar::Text(s) => escape_html(f, s)?,
						TextOrChar::Char(c) => write_html_char(f, c)?,
					}
				}
			}
			TextSpan::Link { link, prefix, .. } => {
				if !text_only {
					f.write_str(r#"<a href=""#)?;
					if prefix.len() > 0 {
						f.write_str(prefix)?;
					}
					escape_html(f, link)?;
					f.write_str(r#"">"#)?;
				}
				escape_html(f, link)?;
				if !text_only {
					f.write_str(r#"</a>"#)?;
				}
			}
		}
	}
	Ok(())
}

fn fmt_block_tags<'a>(block: &Block<'a>, open: bool, f: &mut fmt::Formatter) -> fmt::Result {
	if let Block::Paragraph(text) = block {
		if let Some(false) = text.loose {
			return Ok(());
		} else {
			return if open { write!(f, "<p>") } else { write!(f, "</p>") };
		}
	}

	let no_tag = match block {
		Block::HTML(..) => true,
		_ => false,
	};

	let is_single_tag = match block {
		Block::Break(..) => true,
		_ => false,
	};

	if !no_tag {
		if open {
			write!(f, "<")?;
		} else if !is_single_tag {
			write!(f, "</")?;
		}
	}

	match block {
		Block::BlockQuote(..) => {
			write!(f, "blockquote")?;
		}

		Block::List(list) => {
			if let Some(start) = list.ordered {
				write!(f, "ol")?;
				if open && start > 1 {
					write!(f, " start=\"{}\"", start)?;
				}
			} else {
				write!(f, "ul")?;
			}
		}

		Block::ListItem(item) => {
			if open {
				write!(f, "li")?;
				if let Some(task) = item.task {
					write!(f, "><input type=\"checkbox\"")?;
					if task {
						write!(f, " checked")?;
					}
					write!(f, "/")?;
				}
			} else {
				write!(f, "li")?;
			}
		}

		Block::Break(..) => {
			if open {
				write!(f, "hr")?;
			}
		}

		Block::Header(level, _text) => {
			write!(f, "{}", header(*level))?;
		}

		Block::Paragraph(..) => unreachable!(),

		Block::HTML(_) => {}

		Block::Code(_text) => {
			if open {
				write!(f, "pre><code")?;
			} else {
				write!(f, "code></pre")?;
			}
		}

		Block::FencedCode(code) => {
			if open {
				write!(f, "pre><code")?;
				if let Some(lang) = code.language {
					write!(f, " class=\"language-{}\"", lang)?;
				}
				if let Some(info) = code.info {
					write!(f, " data-info=\"")?;
					escape_html(f, info)?;
					write!(f, "\"")?;
				}
			} else {
				write!(f, "code></pre")?;
			}
		}

		Block::Table(_table) => {
			write!(f, "table")?;
		}

		Block::TableHead(_table) => {
			write!(f, "thead")?;
		}

		Block::TableBody(_table) => {
			write!(f, "tbody")?;
		}

		Block::TableRow(_trow) => {
			write!(f, "tr")?;
		}

		Block::TableCell(cell) => {
			write!(f, "td")?;
			if open {
				cell.align.fmt_attr(f)?;
			}
		}

		Block::TableHeadCell(cell) => {
			write!(f, "th")?;
			if open {
				cell.align.fmt_attr(f)?;
			}
		}
	}

	if !no_tag {
		if is_single_tag {
			if open {
				write!(f, "/>")?;
			}
		} else {
			write!(f, ">")?;
		}
	}

	Ok(())
}

fn header(h: HeaderLevel) -> &'static str {
	match h {
		HeaderLevel::H1 => "h1",
		HeaderLevel::H2 => "h2",
		HeaderLevel::H3 => "h3",
		HeaderLevel::H4 => "h4",
		HeaderLevel::H5 => "h5",
		HeaderLevel::H6 => "h6",
	}
}

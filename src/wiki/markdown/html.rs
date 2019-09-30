use std::fmt;

use super::inline::InlineEvent;
use super::{Block, HeaderLevel, MarkupEvent};

pub fn fmt_html<'a>(event: &MarkupEvent<'a>, f: &mut fmt::Formatter) -> fmt::Result {
	match event {
		MarkupEvent::Inline(span) => {
			for event in span.iter_inline() {
				fmt_inline(&event, f)?;
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
fn write_html_char(f: &mut fmt::Formatter, c: char) -> fmt::Result {
	match c {
		'"' => write!(f, "&quot;"),
		'&' => write!(f, "&amp;"),
		'\'' => write!(f, "&apos;"),
		'<' => write!(f, "&lt;"),
		'>' => write!(f, "&gt;"),
		_ => f.write_str(c.encode_utf8(&mut [0; 4])),
	}
}

fn fmt_inline<'a>(ev: &InlineEvent<'a>, f: &mut fmt::Formatter) -> fmt::Result {
	match ev {
		InlineEvent::Text(s) => f.write_str(s),
		InlineEvent::LineBreak => f.write_str("<br/>"),
		InlineEvent::Entity { entity, .. } => f.write_str(entity),
		InlineEvent::HTML { code, .. } => f.write_str(code),
		_ => panic!("not implemented: HTML for inline {:?}", ev),
	}
}

fn fmt_block_tags<'a>(block: &Block<'a>, open: bool, f: &mut fmt::Formatter) -> fmt::Result {
	if let Block::Paragraph(text) = block {
		if let Some(false) = text.loose {
			return Ok(());
		} else {
			return if open { write!(f, "<p>") } else { write!(f, "</p>") };
		}
	}

	let is_single_tag = match block {
		Block::Break(..) => true,
		_ => false,
	};

	if open {
		write!(f, "<")?;
	} else if !is_single_tag {
		write!(f, "</")?;
	}

	match block {
		Block::BlockQuote(..) => {
			write!(f, "blockquote")?;
		}

		Block::List(list) => {
			if let Some(start) = list.ordered {
				write!(f, "ol")?;
				if open && start > 1 {
					write!(f, " start='{}'", start)?;
				}
			} else {
				write!(f, "ul")?;
			}
		}

		Block::ListItem(item) => {
			if open {
				write!(f, "li")?;
				if let Some(task) = item.task {
					write!(f, "><input type='checkbox'")?;
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
					write!(f, " class='language-{}'", lang)?;
				}
				if let Some(info) = code.info {
					write!(f, " data-info='{}'", info)?;
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

	if is_single_tag {
		if open {
			write!(f, "/>")?;
		}
	} else {
		write!(f, ">")?;
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

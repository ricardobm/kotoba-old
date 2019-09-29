use std::fmt;

use super::{Block, HeaderLevel, MarkupEvent};

pub fn fmt_html<'a>(event: &MarkupEvent<'a>, f: &mut fmt::Formatter) -> fmt::Result {
	match event {
		MarkupEvent::Text(text) => {
			for ch in text.chars() {
				write_html_char(f, ch)?;
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

fn fmt_block_tags<'a>(block: &Block<'a>, open: bool, f: &mut fmt::Formatter) -> fmt::Result {
	if let Block::Paragraph(text) = block {
		if !text.loose {
			return if open { write!(f, "<p>") } else { write!(f, "</p>") };
		} else {
			return Ok(());
		}
	}

	let is_single_tag = match block {
		Block::Break => true,
		_ => false,
	};

	if open {
		write!(f, "<")?;
	} else if !is_single_tag {
		write!(f, "</")?;
	}

	match block {
		Block::BlockQuote => {
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
			let loose = item.is_list_loose();
			if open {
				write!(f, "li")?;
				if loose {
					write!(f, "><p")?;
				}
				if let Some(task) = item.task {
					write!(f, "><input type='checkbox'")?;
					if task {
						write!(f, " checked")?;
					}
				}
			} else {
				if loose {
					write!(f, "p></li")?;
				} else {
					write!(f, "li")?;
				}
			}
		}

		Block::Break => {
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

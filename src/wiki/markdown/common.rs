//! Common utilities for the markdown parser.

/// Default tab width for markdown text.
pub const TAB_WIDTH: usize = 4;

/// Compute the width of the string base indentation and return the
/// column width and byte length.
pub fn indent_width(s: &str, column: usize) -> (usize, usize) {
	let (new_column, bytes) = calc_text_column(s, column, true);
	(new_column - column, bytes)
}

/// Compute the new column after advancing the given text.
pub fn text_column(s: &str, column: usize) -> usize {
	calc_text_column(s, column, false).0
}

fn calc_text_column(s: &str, column: usize, spaces_only: bool) -> (usize, usize) {
	let mut new_column = column;
	let mut bytes = None;
	for (index, chr) in s.char_indices() {
		if chr == '\t' {
			new_column = tab(new_column);
		} else if !spaces_only || (chr.is_whitespace() && chr != '\r' && chr != '\n') {
			new_column += 1;
		} else {
			bytes = Some(index);
			break;
		}
	}
	let bytes = if let Some(bytes) = bytes { bytes } else { s.len() };
	(new_column, bytes)
}

/// Compute the next stop column for a tab at the current position.
#[inline(always)]
pub fn tab(column: usize) -> usize {
	column + tab_width(column)
}

/// Compute the tab width for a tab at the current position.
#[inline(always)]
pub fn tab_width(column: usize) -> usize {
	(TAB_WIDTH - (column % TAB_WIDTH))
}

#[cfg(test)]
pub fn text(input: &str) -> String {
	use regex::Regex;

	lazy_static! {
		static ref RE_EMPTY_LINE: Regex = Regex::new(r#"^[\s&&[^\n\r]]*(\n|\r\n?)"#).unwrap();
		static ref RE_INDENT: Regex = Regex::new(r"^\s*").unwrap();
	}

	let mut base_indent = "";
	let mut text = String::new();
	let mut has_indent = false;
	let input = input.trim_end();
	let input = RE_EMPTY_LINE.replace(input, "");
	for (i, line) in input.lines().enumerate() {
		if !has_indent && line.trim().len() > 0 {
			base_indent = RE_INDENT.find(line).unwrap().as_str();
			has_indent = true;
		}

		let line = if line.starts_with(base_indent) {
			&line[base_indent.len()..]
		} else {
			line
		};
		let line = line.trim_end();
		if i > 0 {
			text.push('\n');
		}
		text.push_str(line);
	}
	text
}

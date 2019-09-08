#[inline]
pub fn get_prefix(s: &str, len: usize) -> &str {
	let end = s.char_indices().map(|x| x.0).nth(len).unwrap_or(s.len());
	&s[..end]
}

#[inline]
pub fn char_in_range(c: char, start: u32, end: u32) -> bool {
	let c = c as u32;
	c >= start && c <= end
}

#[inline]
pub fn hiragana_to_katakana(c: char) -> char {
	use super::constants::*;

	const OFFSET: u32 = KATAKANA_TO_HIRAGANA_OFFSET_SUB;
	const RANGE_START: u32 = KATAKANA_START - OFFSET;
	const RANGE_END: u32 = KATAKANA_TO_HIRAGANA_END - OFFSET;

	if char_in_range(c, RANGE_START, RANGE_END) {
		let code = (c as u32) + OFFSET;
		unsafe { std::char::from_u32_unchecked(code) }
	} else {
		c
	}
}

#[inline]
pub fn is_consonant(c: char, include_y: bool) -> bool {
	match c {
		'b' | 'c' | 'd' | 'f' | 'g' | 'h' | 'j' | 'k' | 'l' | 'm' => true,
		'B' | 'C' | 'D' | 'F' | 'G' | 'H' | 'J' | 'K' | 'L' | 'M' => true,
		'n' | 'p' | 'q' | 'r' | 's' | 't' | 'v' | 'w' | 'x' | 'z' => true,
		'N' | 'P' | 'Q' | 'R' | 'S' | 'T' | 'V' | 'W' | 'X' | 'Z' => true,
		'y' | 'Y' => include_y,
		_ => false,
	}
}

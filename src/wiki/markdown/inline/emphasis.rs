use regex::Regex;
use unicode_categories::UnicodeCategories;

use super::SpanIter;

#[derive(Clone, Debug)]
pub struct Delim<'a> {
	/// The token that opened or closed this delimiter. Note that this is
	/// modified during parsing.
	pub token: &'a str,
	/// Original length of the delimiter run.
	pub length: usize,
	/// Is this left-flanking?
	pub is_lf: bool,
	/// Is this right-flanking?
	pub is_rf: bool,
	/// Is this strikethrough?
	pub is_st: bool,
	/// Can this open an emphasis/strong?
	pub can_open: bool,
	/// Can this close an emphasis/strong?
	pub can_close: bool,
}

impl<'a> Default for Delim<'a> {
	fn default() -> Delim<'a> {
		Delim {
			token:     "",
			length:    0,
			is_lf:     false,
			is_rf:     false,
			is_st:     false,
			can_open:  false,
			can_close: false,
		}
	}
}

pub fn parse_delim<'a>(iter: &mut SpanIter<'a>) -> Option<Delim<'a>> {
	lazy_static! {
		static ref RE_DELIM: Regex = Regex::new(r"^([*]+|[_]+|[~]{2})").unwrap();
	}

	let token = if let Some(m) = RE_DELIM.find(iter.chunk()) {
		m.as_str()
	} else {
		return None;
	};
	let length = token.len();

	let prev = iter.previous_char().unwrap_or(' ');
	iter.skip_bytes(token.len());
	let next = iter.next_char().unwrap_or(' ');

	// Underscore delimiters are not allowed intra-word.
	let is_uc = token.starts_with('_');

	let is_st = token.starts_with('~');

	// A left-flanking delimiter run is a delimiter run that is
	// 1) not followed by Unicode whitespace, and
	// 2) either
	//   a) not followed by a punctuation character, or
	//   b) followed by a punctuation character and preceded by Unicode
	//      whitespace or a punctuation character.
	let is_lf = !is_space(next)
		&& (!is_punctuation(next) || (is_punctuation(next) && (is_space(prev) || is_punctuation(prev))));

	// A right-flanking delimiter run is a delimiter run that is
	// 1) not preceded by Unicode whitespace, and
	// 2) either
	//    a) not preceded by a punctuation character, or
	//    b) preceded by a punctuation character and followed by Unicode
	//    whitespace or a punctuation character.
	let is_rf = !is_space(prev)
		&& (!is_punctuation(prev) || (is_punctuation(prev) && (is_space(next) || is_punctuation(next))));

	let can_open;
	let can_close;
	if is_uc {
		// a single `_` character can open emphasis iff it is part of a
		// left-flanking delimiter run and either (a) not part of a
		// right-flanking delimiter run or (b) part of a right-flanking
		// delimiter run preceded by punctuation
		can_open = is_lf && (!is_rf || is_punctuation(prev));
		// a single `_` character can close emphasis iff it is part of a
		// right-flanking delimiter run and either (a) not part of a
		// left-flanking delimiter run or (b) part of a left-flanking
		// delimiter run followed by punctuation
		can_close = is_rf && (!is_lf || is_punctuation(next));
	} else {
		// a single `*` character can open emphasis iff it is part of a
		// left-flanking delimiter run
		can_open = is_lf;
		// a single `*` character can close emphasis iff it is part of a
		// right-flanking delimiter run
		can_close = is_rf;
	}

	Some(Delim {
		token,
		length,
		is_lf,
		is_rf,
		is_st,
		can_open,
		can_close,
	})
}

fn is_space(c: char) -> bool {
	c.is_whitespace() || c == '\r' || c == '\n'
}

fn is_punctuation(c: char) -> bool {
	c.is_ascii_punctuation() || c.is_punctuation()
}

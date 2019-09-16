mod constants;
mod util;

#[allow(dead_code)]
mod to;
pub use self::to::*;

#[allow(dead_code)]
mod is;
pub use self::is::*;

/// Performs normalization for a search string.
///
/// This performs Unicode normalization (to NFC) and lowercases the input.
///
/// If `as_hiragana` is true, will also convert the input to hiragana.
pub fn normalize_search_string<S: AsRef<str>>(query: S, as_hiragana: bool) -> String {
	use unicode_normalization::UnicodeNormalization;

	// First step, normalize the string. We use NFC to make sure accented
	// characters are a single codepoint.
	let text = query.as_ref().trim().to_lowercase().nfc().collect::<String>();
	let text = if as_hiragana { to_hiragana(text) } else { text };
	text
}

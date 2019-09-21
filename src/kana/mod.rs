mod constants;
mod util;

#[allow(dead_code)]
mod to;
pub use self::to::*;

#[allow(dead_code)]
mod is;
pub use self::is::*;

/// Return `true` if all characters in the string are kanji.
#[allow(dead_code)]
pub fn is_kanji_str<S: AsRef<str>>(s: S) -> bool {
	s.as_ref().chars().all(|chr| is_kanji(chr))
}

/// Return `true` if all characters in the string are either hiragana or katakana.
#[allow(dead_code)]
pub fn is_kana_str<S: AsRef<str>>(s: S) -> bool {
	s.as_ref().chars().all(|chr| is_kana(chr))
}

/// Return `true` if all characters in the string are either kana or kanji.
#[allow(dead_code)]
pub fn is_japanese_text<S: AsRef<str>>(s: S) -> bool {
	s.as_ref().chars().all(|chr| is_kana(chr) || is_kanji(chr))
}

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

//! Japanese character manipulation and conversion module
//!
//! This is largely based on https://github.com/PSeitz/wana_kana_rust but
//! provides an API specifically design for this application.

mod constants;

use std::borrow::Cow;

/// Converts the input string into hiragana. Unknown characters pass-through
/// lowercased.
///
/// Supports mapping romaji and katakana.
#[allow(dead_code)]
pub fn to_hiragana<'a, S>(input: S) -> String
where
	S: Into<Cow<'a, str>>,
{
	use self::constants::*;

	let input = input.into();
	let mut src = input.as_ref();
	let mut out = String::with_capacity(src.len());

	let mut chunk = String::with_capacity(TO_HIRAGANA_MAX_CHUNK * 4);

	while src.len() > 0 {
		let mut chars = src.char_indices();
		let (_, next) = chars.next().unwrap(); // next character
		let (size, _) = chars.next().unwrap_or((src.len(), ' ')); // size of next

		let mut skip = size;
		let mut done = false;

		if char_in_range(next, KATAKANA_START, KATAKANA_TO_HIRAGANA_END) {
			// For katakana we can convert directly just by offseting the code
			let code = (next as u32) - KATAKANA_TO_HIRAGANA_OFFSET_SUB;
			let hiragana = unsafe { std::char::from_u32_unchecked(code) };
			out.push(hiragana);
			done = true;
		} else if !char_in_range(next, HIRAGANA_START, HIRAGANA_END) {
			// Copy the next chunk as lowercase
			chunk.truncate(0);
			chunk.push_str(get_prefix(src, TO_HIRAGANA_MAX_CHUNK));
			unsafe {
				// Fast ASCII-only lowercase conversion just for the sake of
				// comparing to keys in the table.
				let b = chunk.as_bytes_mut();
				for i in 0..b.len() {
					if b[i] >= UPPERCASE_START && b[i] <= UPPERCASE_END {
						b[i] += TO_LOWERCASE_OFFSET_ADD;
					}
				}

				// Double consonant case
				if b.len() >= 2 {
					let c = b[0] as char;
					if c != 'n' && is_consonant(c, true) && b[0] == b[1] {
						out.push('っ');
						done = true;
					}
				}
			}

			if !done {
				// Try to convert all chunk sizes down to 1
				for len in (1..=TO_HIRAGANA_MAX_CHUNK).rev() {
					let chunk = get_prefix(chunk.as_str(), len);
					if let Some(kana) = TO_HIRAGANA.get(chunk) {
						out.push_str(kana);
						skip = chunk.len();
						done = true;
						break;
					}
				}
			}
		}

		// If could not find a conversion, just pass through the character.
		if !done {
			// We want the output lowercased.
			for c in next.to_lowercase() {
				out.push(c);
			}
		}

		src = &src[skip..];
	}

	out
}

#[inline]
fn get_prefix(s: &str, len: usize) -> &str {
	let end = s.char_indices().map(|x| x.0).nth(len).unwrap_or(s.len());
	&s[..end]
}

#[inline]
fn char_in_range(c: char, start: u32, end: u32) -> bool {
	let c = c as u32;
	c >= start && c <= end
}

#[inline]
fn is_consonant(c: char, include_y: bool) -> bool {
	match c {
		'b' | 'c' | 'd' | 'f' | 'g' | 'h' | 'j' | 'k' | 'l' | 'm' => true,
		'n' | 'p' | 'q' | 'r' | 's' | 't' | 'v' | 'w' | 'x' | 'z' => true,
		'y' => include_y,
		_ => false,
	}
}

// spell-checker: disable

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_to_hiragana() {
		check("", "");

		// Katakana
		const H: &str = "ぁあぃいぅうぇえぉおかがきぎくぐけげこごさざしじすずせぜそぞただちぢっつづてでとどなにぬねのはばぱひびぴふぶぷへべぺほぼぽまみむめもゃやゅゆょよらりるれろゎわゐゑをんゔゕゖ";
		const K: &str = "ァアィイゥウェエォオカガキギクグケゲコゴサザシジスズセゼソゾタダチヂッツヅテデトドナニヌネノハバパヒビピフブプヘベペホボポマミムメモャヤュユョヨラリルレロヮワヰヱヲンヴヵヶ";
		check(H, K);
		check(H, H);

		// Romaji
		const D: &str = "しゃぎゃつっじゃあんなん んあんんざ xzm";
		const S: &str = "shyagyatsuxtujaannan n'annza xzm";
		check(D, S);

		// Pass through punctuation
		check("・ー～", "・ー～");

		// Weird katakana
		check("ゔぁ ゔぃ ゔ ゔぇ ゔぉ", "ヷ ヸ ヴ ヹ ヺ");

		// Hepburn style romaji and variation
		check("あーいーうーえーおー", "āīūēō");
		check("あーいーうーえーおー", "âîûêô");

		// Double consonants
		check("ばっば", "babba");
		check("かっか", "cacca");
		check("ちゃっちゃ", "chaccha");
		check("だっだ", "dadda");
		check("ふっふ", "fuffu");
		check("がっが", "gagga");
		check("はっは", "hahha");
		check("じゃっじゃ", "jajja");
		check("かっか", "kakka");
		check("らっら", "lalla");
		check("まっま", "mamma");
		check("なんな", "nanna");
		check("ぱっぱ", "pappa");
		check("くぁっくぁ", "qaqqa");
		check("らっら", "rarra");
		check("さっさ", "sassa");
		check("しゃっしゃ", "shassha");
		check("たった", "tatta");
		check("つっつ", "tsuttsu");
		check("ゔぁっゔぁ", "vavva");
		check("わっわ", "wawwa");
		check("やっや", "yayya");
		check("ざっざ", "zazza");

		// Additional kana tests from wana-kana
		check("おなじ", "onaji");
		check("ぶっつうじ", "buttsuuji");
		check("わにかに", "WaniKani");
		check(
			"わにかに あいうえお 鰐蟹 12345 @#$%",
			"ワニカニ AiUeO 鰐蟹 12345 @#$%",
		);
		check("座禅「ざぜん」すたいる", "座禅‘zazen’スタイル");
		check("ばつげーむ", "batsuge-mu");
	}

	fn check(expected: &str, input: &str) {
		assert_eq!(expected, to_hiragana(input));
		assert_eq!(expected, to_hiragana(input.to_uppercase()));
		assert_eq!(expected, to_hiragana(input.to_lowercase()));
	}
}

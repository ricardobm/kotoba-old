use super::constants::*;
use super::util::*;

/// Returns true for word marks such as `・`, `ー`, `゠`, `ヽ`, `ヾ`, `ゝ`, `ゞ`.
pub fn is_word_mark(chr: char) -> bool {
	match chr {
		'・' | 'ー' | '゠' => true,
		'ヽ' | 'ヾ' | 'ゝ' | 'ゞ' => true, // Katakana and Hiragana Iteration marks
		_ => false,
	}
}

/// Returns true if the character is a hiragana or `ー`.
///
/// Note that this excludes characters from the hiragana block such as the
/// combining diacritics and marks from U+3099 and U+309F.
pub fn is_hiragana(chr: char) -> bool {
	match chr {
		'ゟ' | 'ー' => true,
		_ => char_in_range(chr, HIRAGANA_START, HIRAGANA_END),
	}
}

/// Returns true if the character is a katakana or `ー`.
pub fn is_katakana(chr: char) -> bool {
	match chr {
		'ヿ' | 'ー' => true,
		_ => char_in_range(chr, KATAKANA_START, KATAKANA_END),
	}
}

/// Returns true if the character is a kanji.
pub fn is_kanji(chr: char) -> bool {
	char_in_range(chr, KANJI_START, KANJI_END)
}

/// Returns true if the character is hiragana or katakana.
pub fn is_kana(chr: char) -> bool {
	is_hiragana(chr) || is_katakana(chr)
}

/// Returns true if the character is a japanese-style punctuation.
pub fn is_japanese_punctuation(chr: char) -> bool {
	match chr as u32 {
		// CJK Symbols and Punctuation
		0x3000..=0x303F => true,

		// Katakana punctuation
		0x30FB => true,

		// Kana punctuation
		0xFF61..=0xFF65 => true, // `｡` to `･`

		// Zenkaku punctuation (Halfwidth and Fullwidth Forms)
		0xFF01..=0xFF0F => true,              // `！` to `／`
		0xFF1A..=0xFF1F => true,              // `：` to `？`
		0xFF3B..=0xFF3F => chr != '\u{FF3E}', // `［` to `＿`, but not `＾`
		0xFF5B..=0xFF60 => true,              // `｛` to `｠`

		// Currency symbols
		0xFFE0..=0xFFEE => true,

		_ => false,
	}
}

// spell-checker: disable

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_is_word_mark() {
		let s = "゠ー・ヽヾゝゞ";
		for chr in s.chars() {
			assert!(is_word_mark(chr), "is_word_mark({})", chr);
		}
	}

	#[test]
	fn test_is_hiragana() {
		let s = "ーぁあぃいぅうぇえぉおかがきぎくぐけげこごさざしじすずせぜそぞただちぢっつづてでとどなにぬねのはばぱひびぴふぶぷへべぺほぼぽまみむめもゃやゅゆょよらりるれろゎわゐゑをんゔゕゖゐゑゟ";
		for chr in s.chars() {
			assert!(is_hiragana(chr), "is_hiragana({})", chr);
		}

		for code in 0x3041..=0x3096 {
			let chr = std::char::from_u32(code).unwrap();
			assert!(is_hiragana(chr), "is_hiragana(U+{:04X})", code);
		}

		for chr in "゠・".chars() {
			assert!(!is_hiragana(chr), "!is_hiragana({})", chr);
		}

		assert!(!is_hiragana('\u{3040}'));
		assert!(!is_hiragana('\u{3097}'));
	}

	#[test]
	fn test_is_katakana() {
		let s = "ーァアィイゥウェエォオカガキギクグケゲコゴサザシジスズセゼソゾタダチヂッツヅテデトドナニヌネノハバパヒビピフブプヘベペホボポマミムメモャヤュユョヨラリルレロヮワヰヱヲンヴヵヶヷヸヹヺヿ";
		for chr in s.chars() {
			assert!(is_katakana(chr), "is_katakana({})", chr);
		}

		for code in 0x30A1..=0x30FA {
			let chr = std::char::from_u32(code).unwrap();
			assert!(is_katakana(chr), "is_katakana(U+{:04X})", code);
		}

		for chr in "゠・".chars() {
			assert!(!is_katakana(chr), "!is_katakana({})", chr);
		}

		assert!(!is_katakana('\u{30A0}'));
		assert!(!is_katakana('\u{30FB}'));
	}

	#[test]
	fn test_is_kanji() {
		let s = "一切腹刀丁丂七丄丅丆万丈三上下丌不与丏岐岑岒岓岔岕岖岗岘岙岚岛岜岝岞岟棰棱棲棳棴棵棶棷棸棹棺棻棼棽棾棿龠龡龢龣龤龥龦龧龨龩龪龫龬龭龮龯";
		for chr in s.chars() {
			assert!(is_kanji(chr), "is_kanji({}) -- 0x{:04X}", chr, chr as u32);
		}

		for code in 0x4E00..=0x9FAF {
			let chr = std::char::from_u32(code).unwrap();
			assert!(is_kanji(chr), "is_kanji(U+{:04X})", code);
		}

		assert!(!is_kanji('\u{4DFF}'));
		assert!(!is_kanji('\u{9FB0}'));
	}

	#[test]
	fn test_is_japanese_punctuation() {
		// Japanese punctuation
		let s = "　、。〃〄々〆〇〈〉《》「」『』【】〒〓〔〕〖〗〘〙〚〛〜〝〞〟〠〡〢〣〤〥〦〧〨〩〪〭〮〯〫〬〰〱〲〳〴〵〶〷〸〹〺〻〼〽〾〿・！＂＃＄％＆＇（）＊＋，－．／｡｢｣､･：；＜＝＞？［＼］＿｛｜｝～｟｠｡｢｣､･￠￡￢￣￤￥￦￨￩￪￫￬￭￮";
		for chr in s.chars() {
			assert!(
				is_japanese_punctuation(chr),
				"is_japanese_punctuation({}) -- 0x{:04X}",
				chr,
				chr as u32
			);
		}

		for code in 0x3000..=0x303F {
			let chr = std::char::from_u32(code).unwrap();
			assert!(is_japanese_punctuation(chr), "is_japanese_punctuation(U+{:04X})", code);
		}

		assert!(!is_japanese_punctuation('\u{2FFF}'));
		assert!(!is_japanese_punctuation('\u{3040}'));
		assert!(!is_japanese_punctuation('\u{FF00}'));
		assert!(!is_japanese_punctuation('\u{FFEF}'));
		assert!(!is_japanese_punctuation('ヽ'));
		assert!(!is_japanese_punctuation('ー'));
		assert!(!is_japanese_punctuation('ｚ'));
		assert!(!is_japanese_punctuation('ｦ'));
		assert!(!is_japanese_punctuation('０'));
		assert!(!is_japanese_punctuation('９'));
		assert!(!is_japanese_punctuation('＠'));
		assert!(!is_japanese_punctuation('Ｚ'));
		assert!(!is_japanese_punctuation('＾'));
		assert!(!is_japanese_punctuation('｀'));
		assert!(!is_japanese_punctuation('ｚ'));
		assert!(!is_japanese_punctuation('ヺ'));
		assert!(!is_japanese_punctuation('ￜ'));
	}
}

// This file is taken and modified from
// https://github.com/PSeitz/wana_kana_rust/blob/master/src/constants.rs

use fnv::FnvHashMap;
use fnv::FnvHashSet;

// CharCode References
// http://www.rikai.com/library/kanjitables/kanji_codes.unicode.shtml
// http://unicode-table.com

macro_rules! hashmap {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(hashmap!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { hashmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = hashmap!(@count $($key),*);
            let mut _map = ::fnv::FnvHashMap::with_capacity_and_hasher(_cap, Default::default());
            $(
                let _ = _map.insert($key, $value);
            )*
            _map
        }
    };
}

pub const UPPERCASE_START: u8 = 'A' as u8;
pub const UPPERCASE_END: u8 = 'Z' as u8;
pub const LOWERCASE_START: u8 = 'a' as u8;
pub const TO_LOWERCASE_OFFSET_ADD: u8 = LOWERCASE_START - UPPERCASE_START;

pub const HIRAGANA_START: u32 = 0x3041;
pub const HIRAGANA_END: u32 = 0x3096;
pub const KATAKANA_START: u32 = 0x30A1;
pub const KATAKANA_END: u32 = 0x30FA;

pub const KANJI_START: u32 = 0x4E00;
pub const KANJI_END: u32 = 0x9FAF;

/// Last katakana that can be converted directly to hiragana by offseting.
pub const KATAKANA_TO_HIRAGANA_END: u32 = 0x30F6;
pub const KATAKANA_TO_HIRAGANA_OFFSET_SUB: u32 = KATAKANA_START - HIRAGANA_START;

pub const TO_HIRAGANA_MAX_CHUNK: usize = 4;

// spell-checker: disable

lazy_static! {
	pub static ref TO_HIRAGANA: FnvHashMap<&'static str, &'static str> = hashmap! {
		"." => "。",
		"," => "、",
		": " => "：", // Changed from wana-kana
		":" => "：",  // Changed from wana-kana
		"/" => "・",
		"!" => "！",
		"?" => "？",
		"~" => "〜",
		"-" => "ー",
		"‘" => "「",
		"’" => "」",
		"“" => "『",
		"”" => "』",
		"[" => "［",
		"]" => "］",
		"(" => "（",
		")" => "）",
		"{" => "｛",
		"}" => "｝",

		"a" => "あ",
		"i" => "い",
		"u" => "う",
		"e" => "え",
		"o" => "お",
		"yi" => "い",
		"wu" => "う",
		"whu" => "う",
		"xa" => "ぁ",
		"xi" => "ぃ",
		"xu" => "ぅ",
		"xe" => "ぇ",
		"xo" => "ぉ",
		"xyi" => "ぃ",
		"xye" => "ぇ",
		"ye" => "いぇ",
		"wha" => "うぁ",
		"whi" => "うぃ",
		"whe" => "うぇ",
		"who" => "うぉ",
		"wi" => "うぃ",
		"we" => "うぇ",
		"va" => "ゔぁ",
		"vi" => "ゔぃ",
		"vu" => "ゔ",
		"ve" => "ゔぇ",
		"vo" => "ゔぉ",
		"vya" => "ゔゃ",
		"vyi" => "ゔぃ",
		"vyu" => "ゔゅ",
		"vye" => "ゔぇ",
		"vyo" => "ゔょ",
		"ka" => "か",
		"ki" => "き",
		"ku" => "く",
		"ke" => "け",
		"ko" => "こ",
		"lka" => "ヵ",
		"lke" => "ヶ",
		"xka" => "ヵ",
		"xke" => "ヶ",
		"kya" => "きゃ",
		"kyi" => "きぃ",
		"kyu" => "きゅ",
		"kye" => "きぇ",
		"kyo" => "きょ",
		"ca" => "か",
		"ci" => "き",
		"cu" => "く",
		"ce" => "け",
		"co" => "こ",
		"lca" => "ヵ",
		"lce" => "ヶ",
		"xca" => "ヵ",
		"xce" => "ヶ",
		"qya" => "くゃ",
		"qyu" => "くゅ",
		"qyo" => "くょ",
		"qwa" => "くぁ",
		"qwi" => "くぃ",
		"qwu" => "くぅ",
		"qwe" => "くぇ",
		"qwo" => "くぉ",
		"qa" => "くぁ",
		"qi" => "くぃ",
		"qe" => "くぇ",
		"qo" => "くぉ",
		"kwa" => "くぁ",
		"qyi" => "くぃ",
		"qye" => "くぇ",
		"ga" => "が",
		"gi" => "ぎ",
		"gu" => "ぐ",
		"ge" => "げ",
		"go" => "ご",
		"gya" => "ぎゃ",
		"gyi" => "ぎぃ",
		"gyu" => "ぎゅ",
		"gye" => "ぎぇ",
		"gyo" => "ぎょ",
		"gwa" => "ぐぁ",
		"gwi" => "ぐぃ",
		"gwu" => "ぐぅ",
		"gwe" => "ぐぇ",
		"gwo" => "ぐぉ",
		"sa" => "さ",
		"si" => "し",
		"shi" => "し",
		"su" => "す",
		"se" => "せ",
		"so" => "そ",
		"za" => "ざ",
		"zi" => "じ",
		"zu" => "ず",
		"ze" => "ぜ",
		"zo" => "ぞ",
		"ji" => "じ",
		"sya" => "しゃ",
		"syi" => "しぃ",
		"syu" => "しゅ",
		"sye" => "しぇ",
		"syo" => "しょ",
		"sha" => "しゃ",
		"shu" => "しゅ",
		"she" => "しぇ",
		"sho" => "しょ",
		"shya" => "しゃ", // 4 character code
		"shyu" => "しゅ", // 4 character code
		"shye" => "しぇ", // 4 character code
		"shyo" => "しょ", // 4 character code
		"swa" => "すぁ",
		"swi" => "すぃ",
		"swu" => "すぅ",
		"swe" => "すぇ",
		"swo" => "すぉ",
		"zya" => "じゃ",
		"zyi" => "じぃ",
		"zyu" => "じゅ",
		"zye" => "じぇ",
		"zyo" => "じょ",
		"ja" => "じゃ",
		"ju" => "じゅ",
		"je" => "じぇ",
		"jo" => "じょ",
		"jya" => "じゃ",
		"jyi" => "じぃ",
		"jyu" => "じゅ",
		"jye" => "じぇ",
		"jyo" => "じょ",
		"ta" => "た",
		"ti" => "ち",
		"tu" => "つ",
		"te" => "て",
		"to" => "と",
		"chi" => "ち",
		"tsu" => "つ",
		"ltu" => "っ",
		"xtu" => "っ",
		"tya" => "ちゃ",
		"tyi" => "ちぃ",
		"tyu" => "ちゅ",
		"tye" => "ちぇ",
		"tyo" => "ちょ",
		"cha" => "ちゃ",
		"chu" => "ちゅ",
		"che" => "ちぇ",
		"cho" => "ちょ",
		"cya" => "ちゃ",
		"cyi" => "ちぃ",
		"cyu" => "ちゅ",
		"cye" => "ちぇ",
		"cyo" => "ちょ",
		"chya" => "ちゃ", // 4 character code
		"chyu" => "ちゅ", // 4 character code
		"chye" => "ちぇ", // 4 character code
		"chyo" => "ちょ", // 4 character code
		"tsa" => "つぁ",
		"tsi" => "つぃ",
		"tse" => "つぇ",
		"tso" => "つぉ",
		"tha" => "てゃ",
		"thi" => "てぃ",
		"thu" => "てゅ",
		"the" => "てぇ",
		"tho" => "てょ",
		"twa" => "とぁ",
		"twi" => "とぃ",
		"twu" => "とぅ",
		"twe" => "とぇ",
		"two" => "とぉ",
		"da" => "だ",
		"di" => "ぢ",
		"du" => "づ",
		"de" => "で",
		"do" => "ど",
		"dya" => "ぢゃ",
		"dyi" => "ぢぃ",
		"dyu" => "ぢゅ",
		"dye" => "ぢぇ",
		"dyo" => "ぢょ",
		"dha" => "でゃ",
		"dhi" => "でぃ",
		"dhu" => "でゅ",
		"dhe" => "でぇ",
		"dho" => "でょ",
		"dwa" => "どぁ",
		"dwi" => "どぃ",
		"dwu" => "どぅ",
		"dwe" => "どぇ",
		"dwo" => "どぉ",
		"na" => "な",
		"ni" => "に",
		"nu" => "ぬ",
		"ne" => "ね",
		"no" => "の",
		"nya" => "にゃ",
		"nyi" => "にぃ",
		"nyu" => "にゅ",
		"nye" => "にぇ",
		"nyo" => "にょ",
		"ha" => "は",
		"hi" => "ひ",
		"hu" => "ふ",
		"he" => "へ",
		"ho" => "ほ",
		"fu" => "ふ",
		"hya" => "ひゃ",
		"hyi" => "ひぃ",
		"hyu" => "ひゅ",
		"hye" => "ひぇ",
		"hyo" => "ひょ",
		"fya" => "ふゃ",
		"fyu" => "ふゅ",
		"fyo" => "ふょ",
		"fwa" => "ふぁ",
		"fwi" => "ふぃ",
		"fwu" => "ふぅ",
		"fwe" => "ふぇ",
		"fwo" => "ふぉ",
		"fa" => "ふぁ",
		"fi" => "ふぃ",
		"fe" => "ふぇ",
		"fo" => "ふぉ",
		"fyi" => "ふぃ",
		"fye" => "ふぇ",
		"ba" => "ば",
		"bi" => "び",
		"bu" => "ぶ",
		"be" => "べ",
		"bo" => "ぼ",
		"bya" => "びゃ",
		"byi" => "びぃ",
		"byu" => "びゅ",
		"bye" => "びぇ",
		"byo" => "びょ",
		"pa" => "ぱ",
		"pi" => "ぴ",
		"pu" => "ぷ",
		"pe" => "ぺ",
		"po" => "ぽ",
		"pya" => "ぴゃ",
		"pyi" => "ぴぃ",
		"pyu" => "ぴゅ",
		"pye" => "ぴぇ",
		"pyo" => "ぴょ",
		"ma" => "ま",
		"mi" => "み",
		"mu" => "む",
		"me" => "め",
		"mo" => "も",
		"mya" => "みゃ",
		"myi" => "みぃ",
		"myu" => "みゅ",
		"mye" => "みぇ",
		"myo" => "みょ",
		"ya" => "や",
		"yu" => "ゆ",
		"yo" => "よ",
		"xya" => "ゃ",
		"xyu" => "ゅ",
		"xyo" => "ょ",
		"ra" => "ら",
		"ri" => "り",
		"ru" => "る",
		"re" => "れ",
		"ro" => "ろ",
		"rya" => "りゃ",
		"ryi" => "りぃ",
		"ryu" => "りゅ",
		"rye" => "りぇ",
		"ryo" => "りょ",
		"la" => "ら",
		"li" => "り",
		"lu" => "る",
		"le" => "れ",
		"lo" => "ろ",
		"lya" => "りゃ",
		"lyi" => "りぃ",
		"lyu" => "りゅ",
		"lye" => "りぇ",
		"lyo" => "りょ",
		"wa" => "わ",
		"wo" => "を",
		"lwe" => "ゎ",
		"xwa" => "ゎ",

		//
		// Cases below have been modified from the original
		//

		// Weird katakana and hiragana characters
		"ヷ" => "ゔぁ",
		"ヸ" => "ゔぃ",
		"ヹ" => "ゔぇ",
		"ヺ" => "ゔぉ",
		"ヿ" => "こと", // U+30FF - Katakana Digraph Koto
		"ゟ" => "より", // U+309F - Hiragana Digraph Yori

		// Those `n` cases differ from waka_kana because we don't need IME mode
		"n" => "ん",
		// "nn" => "ん",
		"n'" => "ん", // n" should equal single ん
		"n " => "ん ", // n + space (note the space)
		"xn" => "ん",
		"ltsu" => "っ",  // 4 character code

		// Hepburn style and variations.
		//
		// Note that we replace those using `ー` because the conversion can be
		// ambiguous in those cases.

		"ā" => "あー",
		"ī" => "いー",
		"ū" => "うー",
		"ē" => "えー",
		"ō" => "おー",

		"â" => "あー",
		"î" => "いー",
		"û" => "うー",
		"ê" => "えー",
		"ô" => "おー",

		// Our lowercase implementation for this is limited to ASCII for
		// performance reasons, so repeat for uppercase mappings.
		"Ā" => "あー",
		"Ī" => "いー",
		"Ū" => "うー",
		"Ē" => "えー",
		"Ō" => "おー",

		"Â" => "あー",
		"Î" => "いー",
		"Û" => "うー",
		"Ê" => "えー",
		"Ô" => "おー",
	};

	pub static ref TO_ROMAJI: FnvHashMap<&'static str, &'static str> = hashmap! {
		"　" => " ",
		"！" => "!",
		"？" => "?",
		"。" => ".",
		"：" => ": ", // Changed from wana-kana
		"・" => "/",
		"、" => ",",
		"〜" => "~",
		"ー" => "-",
		"「" => "‘",
		"」" => "’",
		"『" => "“",
		"』" => "”",
		"［" => "[",
		"］" => "]",
		"（" => "(",
		"）" => ")",
		"｛" => "{",
		"｝" => "}",

		"あ" => "a",
		"い" => "i",
		"う" => "u",
		"え" => "e",
		"お" => "o",
		"ゔぁ" => "va",
		"ゔぃ" => "vi",
		"ゔ" => "vu",
		"ゔぇ" => "ve",
		"ゔぉ" => "vo",
		"か" => "ka",
		"き" => "ki",
		"きゃ" => "kya",
		"きぃ" => "kyi",
		"きゅ" => "kyu",
		"く" => "ku",
		"け" => "ke",
		"こ" => "ko",
		"が" => "ga",
		"ぎ" => "gi",
		"ぐ" => "gu",
		"げ" => "ge",
		"ご" => "go",
		"ぎゃ" => "gya",
		"ぎぃ" => "gyi",
		"ぎゅ" => "gyu",
		"ぎぇ" => "gye",
		"ぎょ" => "gyo",
		"さ" => "sa",
		"す" => "su",
		"せ" => "se",
		"そ" => "so",
		"ざ" => "za",
		"ず" => "zu",
		"ぜ" => "ze",
		"ぞ" => "zo",
		"し" => "shi",
		"しゃ" => "sha",
		"しゅ" => "shu",
		"しょ" => "sho",
		"じ" => "ji",
		"じゃ" => "ja",
		"じゅ" => "ju",
		"じょ" => "jo",
		"た" => "ta",
		"ち" => "chi",
		"ちゃ" => "cha",
		"ちゅ" => "chu",
		"ちょ" => "cho",
		"つ" => "tsu",
		"て" => "te",
		"と" => "to",
		"だ" => "da",
		"ぢ" => "di",
		"づ" => "du",
		"で" => "de",
		"ど" => "do",
		"な" => "na",
		"に" => "ni",
		"にゃ" => "nya",
		"にゅ" => "nyu",
		"にょ" => "nyo",
		"ぬ" => "nu",
		"ね" => "ne",
		"の" => "no",
		"は" => "ha",
		"ひ" => "hi",
		"ふ" => "fu",
		"へ" => "he",
		"ほ" => "ho",
		"ひゃ" => "hya",
		"ひゅ" => "hyu",
		"ひょ" => "hyo",
		"ふぁ" => "fa",
		"ふぃ" => "fi",
		"ふぇ" => "fe",
		"ふぉ" => "fo",
		"ば" => "ba",
		"び" => "bi",
		"ぶ" => "bu",
		"べ" => "be",
		"ぼ" => "bo",
		"びゃ" => "bya",
		"びゅ" => "byu",
		"びょ" => "byo",
		"ぱ" => "pa",
		"ぴ" => "pi",
		"ぷ" => "pu",
		"ぺ" => "pe",
		"ぽ" => "po",
		"ぴゃ" => "pya",
		"ぴゅ" => "pyu",
		"ぴょ" => "pyo",
		"ま" => "ma",
		"み" => "mi",
		"む" => "mu",
		"め" => "me",
		"も" => "mo",
		"みゃ" => "mya",
		"みゅ" => "myu",
		"みょ" => "myo",
		"や" => "ya",
		"ゆ" => "yu",
		"よ" => "yo",
		"ら" => "ra",
		"り" => "ri",
		"る" => "ru",
		"れ" => "re",
		"ろ" => "ro",
		"りゃ" => "rya",
		"りゅ" => "ryu",
		"りょ" => "ryo",
		"わ" => "wa",
		"を" => "wo",
		"ん" => "n",

		// Archaic characters
		"ゐ" => "wi",
		"ゑ" => "we",

		// Uncommon character combos
		"きぇ" => "kye",
		"きょ" => "kyo",
		"じぃ" => "jyi",
		"じぇ" => "jye",
		"ちぃ" => "cyi",
		"ちぇ" => "che",
		"ひぃ" => "hyi",
		"ひぇ" => "hye",
		"びぃ" => "byi",
		"びぇ" => "bye",
		"ぴぃ" => "pyi",
		"ぴぇ" => "pye",
		"みぇ" => "mye",
		"みぃ" => "myi",
		"りぃ" => "ryi",
		"りぇ" => "rye",
		"にぃ" => "nyi",
		"にぇ" => "nye",
		"しぃ" => "syi",
		"しぇ" => "she",
		"いぇ" => "ye",
		"うぁ" => "wha",
		"うぉ" => "who",
		"うぃ" => "wi",
		"うぇ" => "we",
		"ゔゃ" => "vya",
		"ゔゅ" => "vyu",
		"ゔょ" => "vyo",
		"すぁ" => "swa",
		"すぃ" => "swi",
		"すぅ" => "swu",
		"すぇ" => "swe",
		"すぉ" => "swo",
		"くゃ" => "qya",
		"くゅ" => "qyu",
		"くょ" => "qyo",
		"くぁ" => "qwa",
		"くぃ" => "qwi",
		"くぅ" => "qwu",
		"くぇ" => "qwe",
		"くぉ" => "qwo",
		"ぐぁ" => "gwa",
		"ぐぃ" => "gwi",
		"ぐぅ" => "gwu",
		"ぐぇ" => "gwe",
		"ぐぉ" => "gwo",
		"つぁ" => "tsa",
		"つぃ" => "tsi",
		"つぇ" => "tse",
		"つぉ" => "tso",
		"てゃ" => "tha",
		"てぃ" => "thi",
		"てゅ" => "thu",
		"てぇ" => "the",
		"てょ" => "tho",
		"とぁ" => "twa",
		"とぃ" => "twi",
		"とぅ" => "twu",
		"とぇ" => "twe",
		"とぉ" => "two",
		"ぢゃ" => "dya",
		"ぢぃ" => "dyi",
		"ぢゅ" => "dyu",
		"ぢぇ" => "dye",
		"ぢょ" => "dyo",
		"でゃ" => "dha",
		"でぃ" => "dhi",
		"でゅ" => "dhu",
		"でぇ" => "dhe",
		"でょ" => "dho",
		"どぁ" => "dwa",
		"どぃ" => "dwi",
		"どぅ" => "dwu",
		"どぇ" => "dwe",
		"どぉ" => "dwo",
		"ふぅ" => "fwu",
		"ふゃ" => "fya",
		"ふゅ" => "fyu",
		"ふょ" => "fyo",

		//  Small Characters (normally not transliterated alone)
		"ぁ" => "a",
		"ぃ" => "i",
		"ぇ" => "e",
		"ぅ" => "u",
		"ぉ" => "o",
		"ゃ" => "ya",
		"ゅ" => "yu",
		"ょ" => "yo",
		"っ" => "~tsu",  //to detect when we fail to handle this case
		"ゕ" => "ka",
		"ゖ" => "ka",
		"ゎ" => "wa",

		// Ambiguous consonant vowel pairs
		"んあ" => "n'a",
		"んい" => "n'i",
		"んう" => "n'u",
		"んえ" => "n'e",
		"んお" => "n'o",
		"んや" => "n'ya",
		"んゆ" => "n'yu",
		"んよ" => "n'yo",

		// Ambiguous consonant vowel pairs with digraphs
		"んうぁ" => "nwha",
		"んうぉ" => "nwho",
		"んうぃ" => "nwi",
		"んうぇ" => "nwe",
		"んいぇ" => "n'ye",
	};

	/// First characters of all keys in the [TO_ROMAJI] table.
	pub static ref TO_ROMAJI_CHARS: FnvHashSet<char> = {
		let mut set = FnvHashSet::<char>::default();
		for key in TO_ROMAJI.keys() {
			let first = key.chars().next().unwrap();
			set.insert(first);
		}
		set
	};

	/// Maximum length of any key in the [TO_ROMAJI] table.
	pub static ref TO_ROMAJI_MAX_CHUNK: usize = {
		let mut size = 0;
		for key in TO_ROMAJI.keys() {
			size = std::cmp::max(size, key.chars().count());
		}
		size
	};
}

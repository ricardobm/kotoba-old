//! Japanese sentence parsing

use regex::Regex;

use super::dictionary::Form;
use kana;

pub struct ParsedResult {
	pub sentence: Vec<SentenceItem>,
	pub analysis: Vec<Token>,
}

pub enum SentenceItem {
	Text(String),
	Word(Meaning),
}

pub struct Meaning {
	/// Source text.
	text: String,
	/// De-inflected form for the text.
	dict: String,
	/// Meanings found for the text.
	list: Vec<Form>,
	/// Provide de-inflection information for this part.
	info: Vec<String>,
}

/// Kind of elements in a sentence.
#[derive(PartialEq, Eq)]
pub enum Kind {
	/// Punctuation, spaces, and any other unsupported text.
	Text,
	/// Romaji or english text.
	Romaji,
	/// Numeric value, date, etc.
	Number,
	/// Kana-only text, either hiragana or katakana.
	Kana,
	/// Kanji-only text.
	Kanji,
	/// Mixed kana and kanji text.
	Mixed,
}

impl std::fmt::Display for Kind {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let s = match self {
			Kind::Text => "text",
			Kind::Romaji => "romaji",
			Kind::Number => "number",
			Kind::Kana => "kana",
			Kind::Kanji => "kanji",
			Kind::Mixed => "mixed",
		};
		write!(f, "{}", s)
	}
}

/// Token is the smallest unit for a parsed text element.
pub struct Token {
	/// Kind of token.
	pub kind: Kind,

	/// Raw text for this token.
	pub text: String,

	/// The text matched by the morphological analysis. This can differ from
	/// the raw text.
	pub surface: String,

	/// Dictionary form for the unit (attempt).
	pub dict: String,

	/// Kana reading for this unit, if available.
	pub reading: String,

	/// Parts of speech.
	pub parts: Vec<String>,

	/// Conjugation form name.
	pub conjugation: String,

	/// Inflection form name.
	pub inflection: String,
}

impl Default for Token {
	fn default() -> Token {
		Token {
			kind:        Kind::Text,
			text:        Default::default(),
			surface:     Default::default(),
			dict:        Default::default(),
			reading:     Default::default(),
			parts:       Default::default(),
			conjugation: Default::default(),
			inflection:  Default::default(),
		}
	}
}

impl std::fmt::Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{} '{}'", self.kind, self.text)?;

		if self.surface.len() > 0 && self.surface != self.text {
			write!(f, "='{}'", self.surface)?;
		}

		let has_dict = self.dict.len() > 0 && self.dict != self.text && self.dict != self.surface;
		let has_read = self.reading.len() > 0
			&& self.reading != self.text
			&& self.reading != self.surface
			&& self.reading != self.dict;

		if has_dict || has_read {
			write!(f, " (")?;
			if has_dict {
				write!(f, "{}", self.dict)?;
				if has_read {
					write!(f, " / {}", self.reading)?;
				}
			} else {
				write!(f, "{}", self.reading)?;
			}
			write!(f, ")")?;
		}

		if self.parts.len() > 0 {
			write!(f, " [{}]", self.parts.join(" "))?;
		}

		if self.conjugation.len() > 0 || self.inflection.len() > 0 {
			write!(f, " {{")?;
			if self.conjugation.len() > 0 {
				write!(f, "{}", self.conjugation)?;
				if self.inflection.len() > 0 {
					write!(f, " / {}", self.inflection)?;
				}
			} else {
				write!(f, "{}", self.inflection)?;
			}
			write!(f, "}}")?;
		}

		Ok(())
	}
}

#[allow(dead_code)]
pub fn parse_tokens(sentence: &str) -> Vec<Token> {
	let mut tokens = Vec::new();

	// Perform a morphological analysis on the sentence:

	use yoin::ipadic::tokenizer;

	let mut items = Vec::new();
	let tokenizer = tokenizer();
	for token in tokenizer.tokenize(sentence) {
		items.push(AnalysisItem::from_token(token, sentence));
	}

	// Group into tokens:

	let last = items.len() - 1;
	let mut cur = 0;
	while cur <= last {
		let item = &items[cur];
		let mut next = cur + 1;

		let mut token = if cur < last && (item.txt == "+" || item.txt == "-") && is_number_str(&items[next].txt) {
			// Current token is a signed number
			let token = Token {
				kind: Kind::Number,
				text: format!("{}{}", item.txt, items[next].txt),
				..Default::default()
			};
			next += 1;
			token
		} else if is_number_str(&item.txt) {
			// Current token is a number
			Token {
				kind: Kind::Number,
				text: item.txt.clone(),
				..Default::default()
			}
		} else if is_symbol_str(&item.txt) {
			Token {
				kind: Kind::Text,
				text: item.txt.clone(),
				..Default::default()
			}
		} else {
			// Token is a word
			Token {
				kind:        word_kind(&item.txt),
				text:        item.txt.clone(),
				surface:     item.surface.clone(),
				dict:        item.dictionary.clone(),
				reading:     item.pronunciation.clone(),
				conjugation: item.conjugation.clone(),
				inflection:  item.inflection.clone(),

				parts: {
					let mut parts = Vec::new();
					if item.part_of_speech.len() > 0 {
						parts.push(item.part_of_speech.clone());
						if item.part_of_speech_1.len() > 0 {
							parts.push(item.part_of_speech_1.clone());
							if item.part_of_speech_2.len() > 0 {
								parts.push(item.part_of_speech_2.clone());
								if item.part_of_speech_3.len() > 0 {
									parts.push(item.part_of_speech_3.clone());
								}
							}
						}
					}
					parts
				},
			}
		};

		// Merge number with separators as a single token
		if token.kind == Kind::Number {
			while next <= last - 1 && is_number_separator(&items[next].txt) && is_number_str(&items[next + 1].txt) {
				token.text.push_str(&items[next].txt);
				token.text.push_str(&items[next + 1].txt);
				next += 2;
			}
		}

		tokens.push(token);

		cur = next;
	}

	// Merge sequence of symbols
	let iter = tokens.into_iter();
	let mut tokens: Vec<Token> = Vec::new();
	for it in iter {
		let count = tokens.len();
		if it.kind == Kind::Text && count > 0 && tokens[count - 1].kind == Kind::Text {
			tokens[count - 1].text.push_str(&it.text);
		} else {
			tokens.push(it);
		}
	}

	tokens
}

struct AnalysisItem {
	/// Byte offset for the token start.
	pub pos: usize,
	/// Byte offset for the token end.
	pub end: usize,
	/// Raw text for the token.
	pub txt: String,

	// Features:
	/// Surface is the actual text considered by the morphological analysis.
	pub surface: String,
	/// Main part of speech.
	pub part_of_speech: String,
	/// Additional part of speech.
	pub part_of_speech_1: String,
	/// Additional part of speech.
	pub part_of_speech_2: String,
	/// Additional part of speech.
	pub part_of_speech_3: String,
	/// Conjugation name, if available.
	pub conjugation: String,
	/// Inflection name, if available.
	pub inflection: String,
	/// This is the dictionary form for the term, if available.
	pub dictionary: String,
	/// Kana pronunciation for the term, if available.
	pub pronunciation: String,
}

impl AnalysisItem {
	fn from_token<'a>(token: yoin::tokenizer::Token<'a>, input: &'a str) -> AnalysisItem {
		let (pos, end) = (token.start(), token.end());
		let mut features = token.features();
		let txt = &input[pos..end];
		AnalysisItem {
			pos: pos,
			end: end,
			txt: txt.to_string(),

			surface: token.surface().to_string(),

			// Feature fields are, in order¹:
			//
			// - Part of Speech,
			// - Part of Speech section 1,
			// - Part of Speech section 2,
			// - Part of Speech section 3,
			// - Conjugated form,
			// - Inflection,
			// - Reading (dictionary form),
			// - Pronunciation (kana pronunciation)
			//
			// [1] - Based on https://github.com/jordwest/mecab-docs-en
			part_of_speech:   Self::feature(&mut features),
			part_of_speech_1: Self::feature(&mut features),
			part_of_speech_2: Self::feature(&mut features),
			part_of_speech_3: Self::feature(&mut features),
			conjugation:      Self::feature(&mut features),
			inflection:       Self::feature(&mut features),
			dictionary:       kana::to_hiragana(Self::feature(&mut features)),
			pronunciation:    kana::to_hiragana(Self::feature(&mut features)),
		}
	}

	fn feature<'s, F: Iterator<Item = &'s str>>(features: &mut F) -> String {
		match features.next() {
			Some("*") => String::new(),
			Some(val) => String::from(val),
			_ => String::new(),
		}
	}
}

//
// Helper char functions
//

fn word_kind(s: &str) -> Kind {
	if kana::is_kana_str(s) {
		Kind::Kana
	} else if kana::is_kanji_str(s) {
		Kind::Kanji
	} else if kana::has_japanese_text(s) {
		Kind::Mixed
	} else {
		Kind::Romaji
	}
}

fn is_number_separator(s: &str) -> bool {
	lazy_static! {
		static ref RE_SEPARATOR: Regex = Regex::new(r"^([-.,_·/:])$").unwrap();
	}
	RE_SEPARATOR.is_match(s)
}

fn is_number_str(s: &str) -> bool {
	s.chars().all(|c| c.is_numeric())
}

fn is_symbol_str(s: &str) -> bool {
	s.chars().all(|c| is_symbol(c))
}

fn is_symbol(chr: char) -> bool {
	if chr.is_whitespace() {
		true
	} else if kana::is_japanese_punctuation(chr) {
		true
	} else if chr.is_control() {
		true
	} else {
		!(chr.is_alphanumeric() || kana::is_kana(chr) || kana::is_kanji(chr) || kana::is_word_mark(chr))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_tokenization() {
		fn compare(sentence: &str, expected: Vec<&'static str>) {
			let tokens = parse_tokens(sentence);
			let tokens = tokens.into_iter().map(|x| format!("{}", x)).collect::<Vec<_>>();
			if expected.len() != tokens.len() {
				assert!(
					false,
					"expected {} tokens, was {}\nEXPECTED:\n  {}\nACTUAL:\n  {}",
					expected.len(),
					tokens.len(),
					expected.join("\n  "),
					tokens.join("\n  "),
				);
			}

			for i in 0..tokens.len() {
				let actual = &tokens[i];
				let expected = expected[i];
				assert_eq!(
					expected,
					actual,
					"at #{}, expected `{}` was `{}`",
					i + 1,
					expected,
					actual
				);
			}
		}

		compare(
			"sentence x123: 「あれがデネブ、アルタイル、ベガ」 真っ暗な世界から見上げた +1,345_789.45 -1 123",
			vec![
				"romaji 'sentence' [名詞 固有名詞 組織]",
				"text ' '",
				"romaji 'x' [名詞 一般]",
				"number '123'",
				"text ': 「'",
				"kana 'あれ' [名詞 代名詞 一般]",
				"kana 'が' [助詞 格助詞 一般]",
				"kana 'デネブ'='デネル' (でねる) [名詞 固有名詞 組織]",
				"text '、'",
				"kana 'アル' (ある) [名詞 固有名詞 人名 名]",
				"kana 'タイル' (たいる) [名詞 一般]",
				"text '、'",
				"kana 'ベガ' [名詞 一般]",
				"text '」 '",
				"mixed '真っ暗' (まっくら) [名詞 形容動詞語幹]",
				"kana 'な' (だ) [助動詞] {特殊・ダ / 体言接続}",
				"kanji '世界' (せかい) [名詞 一般]",
				"kana 'から' [助詞 格助詞 一般]",
				"mixed '見上げ' (見上げる / みあげ) [動詞 自立] {一段 / 連用形}",
				"kana 'た' [助動詞] {特殊・タ / 基本形}",
				"text ' '",
				"number '+1,345_789.45'",
				"text ' '",
				"number '-1'",
				"text ' '",
				"number '123'",
			],
		);
	}
}

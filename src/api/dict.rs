//! Dictionary search API

use rocket::State;
use rocket_contrib::json::Json;

use app::App;
use japanese;
use japanese::dictionary::{Response, SearchArgs, Source};
use logging::RequestLog;

#[post("/dict/search", data = "<input>")]
pub fn search(log: RequestLog, input: Json<SearchArgs>, app: State<&App>) -> Json<Response> {
	let dict = app.dictionary();
	Json(dict.query(&log, input.into_inner()))
}

#[get("/dict/tags")]
pub fn tags(app: State<&App>) -> Json<Vec<Source>> {
	let dict = app.dictionary();
	Json(dict.tags())
}

#[get("/dict/analyze?<q>")]
pub fn analyze(q: String) -> Json<Vec<AnalysisToken>> {
	use yoin::ipadic::tokenizer;

	let mut out = Vec::new();
	let input = q.as_str();
	let tokenizer = tokenizer();
	for token in tokenizer.tokenize(input).into_iter().collect::<Vec<_>>() {
		out.push(AnalysisToken::from_token(token, input));
	}

	Json(out)
}

#[derive(Serialize)]
pub struct AnalysisToken {
	/// Original form
	pub surface: String,
	/// Byte offset for the token start.
	pub pos: usize,
	/// Byte offset for the token end.
	pub end: usize,
	/// Raw text for the token.
	pub txt: String,

	// Features:
	pub part_of_speech:   String,
	pub part_of_speech_1: String,
	pub part_of_speech_2: String,
	pub part_of_speech_3: String,
	pub conjugated_form:  String,
	pub inflection:       String,
	pub reading:          String,
	pub pronunciation:    String,

	pub possible_inflections: Vec<japanese::Inflection>,
}

impl AnalysisToken {
	fn from_token<'a>(token: yoin::tokenizer::Token<'a>, input: &str) -> AnalysisToken {
		use kana;

		let (pos, end) = (token.start(), token.end());
		let mut features = token.features();
		let text = &input[pos..end];
		AnalysisToken {
			pos:     pos,
			end:     end,
			txt:     String::from(text),
			surface: token.surface().to_string(),

			// Feature fields are, in order¹:
			//
			// - Part of Speech,
			// - Part of Speech section 1,
			// - Part of Speech section 2,
			// - Part of Speech section 3,
			// - Conjugated form,
			// - Inflection,
			// - Reading,
			// - Pronunciation
			//
			// [1] - Based on https://github.com/jordwest/mecab-docs-en
			part_of_speech:   Self::feature(&mut features),
			part_of_speech_1: Self::feature(&mut features),
			part_of_speech_2: Self::feature(&mut features),
			part_of_speech_3: Self::feature(&mut features),
			conjugated_form:  Self::feature(&mut features),
			inflection:       Self::feature(&mut features),
			reading:          kana::to_hiragana(Self::feature(&mut features)),
			pronunciation:    kana::to_hiragana(Self::feature(&mut features)),

			possible_inflections: if kana::has_japanese_text(text) {
				japanese::deinflect(text)
			} else {
				Default::default()
			}
		}
	}

	fn feature<'a, 's, F: Iterator<Item = &'s str>>(features: &mut F) -> String {
		match features.next() {
			Some("*") => String::new(),
			Some(val) => String::from(val),
			_ => String::new(),
		}
	}
}

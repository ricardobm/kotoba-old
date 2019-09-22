use std::collections::HashMap;
use std::io::Read;
use std::time::Duration;

use slog::Logger;

use reqwest::{Client, Url};
use scraper::{Html, Selector};

use super::audio_helper::*;
use crate::kana::to_hiragana;
use crate::util;
use crate::util::check_response;

const DEFAULT_TIMEOUT_MS: u64 = 5000;

pub struct Args {
	/// Term to lookup.
	pub term: String,

	/// If `true`, only look the 20,000 most common words.
	pub common: bool,

	/// If `true`, allow vulgar terms in the results.
	pub vulgar: bool,

	/// If `true`, match the start of a word instead of exactly.
	pub starts: bool,
}

impl Default for Args {
	fn default() -> Args {
		Args {
			term:   String::new(),
			common: false,
			vulgar: false,
			starts: false,
		}
	}
}

#[derive(Serialize)]
pub struct Entry {
	/// Main japanese term.
	pub term: String,

	/// Kana reading of the term.
	pub kana: String,

	/// Audio URLs.
	pub audio: Vec<String>,

	/// English definition for the term.
	pub english: String,

	/// Additional information to appear after the english definition.
	///
	/// This would appear rendered as italicized text in gray after the
	/// definition.
	pub info: Vec<String>,

	/// Order of this entry in the results.
	pub order: usize,
}

/// Query `japanesepod101.com` dictionary.
pub fn query_dictionary(log: &Logger, args: Args) -> util::Result<Vec<Entry>> {
	lazy_static! {
		static ref SEL_RESULT_ROW: Selector = Selector::parse("div.dc-result-row").unwrap();
		static ref SEL_TERM_ELEM: Selector = Selector::parse("span.dc-vocab").unwrap();
		static ref SEL_KANA_ELEM: Selector = Selector::parse("span.dc-vocab_kana").unwrap();
		static ref SEL_AUDIO_SRC: Selector = Selector::parse("div.di-player:first-of-type audio > source").unwrap();
		static ref SEL_ENGLISH_ELEM: Selector = Selector::parse("span.dc-english").unwrap();
		static ref SEL_ENGLISH_IS_INFO_ELEM: Selector = Selector::parse("span.dc-english-grey").unwrap();
	}

	let mut params = HashMap::new();
	params.insert("search_query", args.term.as_str());
	params.insert("post", "dictionary_reference");
	params.insert("match_type", if args.starts { "starts" } else { "exact" });
	if args.vulgar {
		params.insert("vulgar", "true");
	}
	if args.common {
		params.insert("common", "true");
	}

	let client = Client::builder()
		.timeout(Duration::from_millis(DEFAULT_TIMEOUT_MS))
		.build()?;

	let mut response = client
		.post("https://www.japanesepod101.com/learningcenter/reference/dictionary_post")
		.form(&params)
		.send()?;

	check_response(log, &response)?;

	let mut doc = String::new();
	response.read_to_string(&mut doc)?;

	let mut out = Vec::new();
	let doc = Html::parse_document(doc.as_str());

	// Iterate over every result row.
	for (order, row) in doc.select(&SEL_RESULT_ROW).enumerate() {
		// Main term for the result row
		let term = match row.select(&SEL_TERM_ELEM).next() {
			Some(el) => el.text().collect::<String>().trim().to_string(),
			_ => continue,
		};
		if term.len() == 0 {
			continue;
		}

		// Kana reading for the result row
		let kana = match row.select(&SEL_KANA_ELEM).next() {
			Some(el) => to_hiragana(el.text().collect::<String>().trim()),
			_ => String::new(),
		};

		let (english, info) = if let Some(el) = row.select(&SEL_ENGLISH_ELEM).next() {
			let mut english = el.text().collect::<String>();
			let info = el
				.select(&SEL_ENGLISH_IS_INFO_ELEM)
				.map(|e| e.text().collect::<String>().trim().to_string())
				.collect::<Vec<_>>();
			for it in info.iter() {
				english = english.replace(it.as_str(), "");
			}
			english = english.trim().to_string();
			(english, info)
		} else {
			Default::default()
		};

		let mut audio = Vec::new();
		for el in row.select(&SEL_AUDIO_SRC) {
			let src = el.value().attr("src").unwrap_or("").trim();
			if src.len() > 0 {
				audio.push(src.to_string());
			}
		}

		out.push(Entry {
			term,
			kana,
			english,
			info,
			order,
			audio,
		});
	}

	Ok(out)
}

/// Load audio pronunciations from `japanesepod101.com` results.
pub fn load_dictionary_pronunciations(sink: AudioSink, kanji: &str, kana: &str) {
	let result = query_dictionary(&sink.log, Args {
		term: kanji.to_string(),
		vulgar: true,
		..Default::default()
	});

	match result {
		Ok(result) => {
			let audio_urls = result
				.into_iter()
				.filter(|x| &x.kana == kana)
				.map(|x| x.audio)
				.flatten()
				.collect::<Vec<_>>();
			load_audio_list(sink, audio_urls);
		}
		Err(err) => {
			sink.send_err(err);
		}
	}

}

/// Load audio pronunciation from `languagepod101.com`.
pub fn load_pronunciation(sink: AudioSink, kanji: &str, kana: &str) {
	match do_load_pronunciation(&sink.log, kanji, kana) {
		Err(err) => {
			sink.send_err(err);
		}
		Ok(Some(data)) => {
			sink.send_data(data);
		}
		Ok(None) => {}
	}
}

fn do_load_pronunciation(log: &Logger, kanji: &str, kana: &str) -> util::Result<Option<crate::audio::AudioData>> {
	const BLACKLIST_HASH: &str = "ae6398b5a27bc8c0a771df6c907ade794be15518174773c58c7c7ddd17098906";

	let mut url = Url::parse("https://assets.languagepod101.com/dictionary/japanese/audiomp3.php")?;
	url.query_pairs_mut()
		.append_pair("kanji", kanji)
		.append_pair("kana", kana);

	let audio = do_load_audio(log, url)?;
	if &audio.hash == BLACKLIST_HASH {
		Ok(None)
	} else {
		Ok(Some(audio))
	}
}
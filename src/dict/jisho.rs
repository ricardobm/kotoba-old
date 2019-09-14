use std::io::Read;
use std::time::Duration;

use reqwest::{Client, Url};

use regex::Regex;

use itertools::*;

use scraper::{ElementRef, Html, Selector};
use selectors::attr::CaseSensitivity;

use super::audio::*;
use crate::kana::{is_kanji, to_hiragana};
use crate::util;
use crate::util::check_response;

const DEFAULT_TIMEOUT_MS: u64 = 3500;

/// Load audio pronunciations from `jisho.org` results.
pub fn load_pronunciations(kanji: &str, kana: &str) -> util::Result<Vec<AudioResult>> {
	lazy_static! {
		static ref SEL_AUDIO: Selector = Selector::parse("audio").unwrap();
		static ref SEL_TEXT: Selector = Selector::parse("span.text").unwrap();
		static ref SEL_FURIGANA: Selector = Selector::parse("span.furigana").unwrap();
		static ref SEL_SPAN: Selector = Selector::parse("span").unwrap();
		static ref SEL_SOURCE: Selector = Selector::parse("source").unwrap();
		static ref RE_MP3: Regex = Regex::new(r"(?i)\.mp3$").unwrap();
	}

	let mut url = Url::parse("https://jisho.org/search/")?;
	url.query_pairs_mut().append_pair("keyword", kanji);

	let client = Client::builder()
		.timeout(Duration::from_millis(DEFAULT_TIMEOUT_MS))
		.build()?;

	let mut response = client.get(url).send()?;
	check_response(&response)?;

	let mut doc = String::new();
	response.read_to_string(&mut doc)?;

	let doc = Html::parse_document(doc.as_str());

	struct Item {
		term: String,
		kana: String,
		src:  String,
	}

	let mut results = Vec::new();

	'outer: for audio in doc.select(&SEL_AUDIO) {
		// Look for the closest `div.concept_light-wrapper`
		let mut wrapper = audio.parent();
		while let Some(current) = wrapper {
			let val = match current.value().as_element() {
				Some(v) => v,
				_ => continue 'outer,
			};
			if val.name() == "div" && val.has_class("concept_light-wrapper", CaseSensitivity::AsciiCaseInsensitive) {
				break;
			}
			wrapper = current.parent();
		}

		let wrapper = match wrapper {
			Some(v) => ElementRef::wrap(v).unwrap(),
			_ => continue,
		};

		// Extract the text for the entry.
		let text = match wrapper.select(&SEL_TEXT).next() {
			Some(v) => v.text().collect::<String>().trim().to_string(),
			_ => continue,
		};

		// Extracts the furigana for the entry, so we can generate the kana
		// reading.
		//
		// The furigana field has one `<span>` per character in the entry
		// text. Some of those spans will contain a reading, others will be
		// empty. The empty spans are either for kana or for kanji that have
		// no reading (in the latter case, the reading is usually merged with
		// the kanji before it).
		let furigana = match wrapper.select(&SEL_FURIGANA).next() {
			Some(v) => v
				.select(&SEL_SPAN)
				.map(|x| x.text().collect::<String>().trim().to_string())
				.collect::<Vec<_>>(),
			_ => continue,
		};

		let char_count = text.chars().count();
		if char_count != furigana.len() {
			eprintln!(
				"WARNING: Jisho furigana for {} does not match text length ({} != {})",
				text,
				char_count,
				furigana.len()
			);
			continue;
		}

		// Replace each kanji in the original text by its furigana reading.
		let kana = furigana
			.into_iter()
			.zip(text.chars())
			.map(|(furigana, chr)| if is_kanji(chr) { furigana } else { chr.to_string() })
			.join("");
		let kana = to_hiragana(kana);

		let src = audio
			.select(&SEL_SOURCE)
			.map(|x| x.value().attr("src").unwrap_or(""))
			.filter(|x| RE_MP3.is_match(x))
			.next();
		let src = match src {
			Some(v) => v,
			_ => continue,
		};

		let src = if src.starts_with("//") {
			format!("https:{}", src)
		} else {
			src.to_string()
		};

		results.push(Item {
			term: text,
			kana: kana,
			src:  src,
		});
	}

	let audio_urls = results
		.into_iter()
		.filter(|x| &x.kana == kana && &x.term == kanji)
		.map(|x| x.src)
		.collect::<Vec<_>>();
	Ok(load_audio_list(audio_urls))
}

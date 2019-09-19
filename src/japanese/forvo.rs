use std::io::Read;
use std::thread;
use std::time::Duration;

use slog::Logger;

use crossbeam::channel::unbounded;
use itertools::*;
use percent_encoding;
use regex::Regex;
use reqwest::{Client, Url};
use scraper::{ElementRef, Html, Selector};

use super::audio_helper::*;
use crate::db::search_strings;
use crate::util;
use crate::util::check_response;

const DEFAULT_TIMEOUT_MS: u64 = 5000;

/// Load audio pronunciations from `forvo.com` results.
pub fn load_pronunciations(sink: AudioSink, kanji: &str, kana: &str) {
	if let Err(err) = do_load_pronunciations(sink.clone(), kanji, kana) {
		sink.send_err(err);
	}
}

fn do_load_pronunciations(sink: AudioSink, kanji: &str, kana: &str) -> util::Result<()> {
	// Since we can only lookup for the main term, Forvo search will return
	// a lot of unrelated items.
	//
	// For example, searching for `明日` will return
	//
	//     - 明日
	//     - また 明日
	//     - 明日 (あした、あす、みょうにち）
	//     - 明日（あす）
	//     - 明日、会議を行う予定だ。
	//     - 明日（あした）
	//
	// along with lots of phrases.
	//
	// We need to filter those that down to the proper matches. For example,
	// when searching for `あした` we would want to return the
	//
	//     - `明日 (あした、あす、みょうにち）`, and
	//     - `明日（あした）`
	//
	// entries. At the same time we don't want to eliminate entries such as
	// `明日`, because many times the readings are not included as part of
	// the text.
	//
	// Before filtering, we use `search_strings` to normalize the text, and
	// split and filter out only the filterable japanese words in the text.
	//
	// After that we compare both the kanji and kana keys to the returned terms.
	//
	// NOTE: we also split terms and kana to support searching by phrases.

	let keys_term = search_strings(kanji);
	let keys_kana = search_strings(kana);

	fn is_match(text: &Vec<String>, keys_term: &Vec<String>, keys_kana: &Vec<String>) -> bool {
		// When matching the kanji, all words must be in the matched text.
		let match_term = keys_term.iter().all(|x| text.contains(x));

		// Same for matching the kana.
		let match_kana = keys_kana.len() > 0 && keys_kana.iter().all(|x| text.contains(x));

		// We prioritize matching the kana over the kanji, unless:
		// - the query does not include the desired kana reading; then we match
		//   only with the kanji part;
		// - there is a kana reading, but the text matches exactly the kanji
		//   terms (meaning this is an entry that has no reading).
		let is_kanji = keys_kana.len() == 0 || text.iter().all(|x| keys_term.contains(x));
		match_kana || (is_kanji && match_term)
	}

	// First we scrape the main search index page for the top results. We are
	// actually interested in loading from the details page, but those are
	// linked by the main entries.

	let param = percent_encoding::utf8_percent_encode(kanji, percent_encoding::NON_ALPHANUMERIC);
	let url = format!("https://forvo.com/search/{}/ja", param);
	let index_results = scrape_page(&sink.log, &url)?;

	const MAX_WORKERS: usize = 10;
	const MAX_INDEX_RESULTS: usize = 10;
	const MAX_DETAIL_RESULTS: usize = 4;

	// Filter matching entries and collect the top results from the index:

	let index_results = index_results
		.into_iter()
		.filter(|x| is_match(&x.text, &keys_term, &keys_kana))
		.take(MAX_INDEX_RESULTS)
		.collect::<Vec<_>>();

	// Start workers to scrape the detail pages:

	type WorkerResult = util::Result<Vec<ForvoEntry>>;

	let (tx_work, rx_work) = unbounded::<ForvoEntry>();
	let (tx_data, rx_data) = unbounded::<WorkerResult>();

	let num_workers = std::cmp::min(MAX_WORKERS, index_results.len());
	let mut handles = Vec::new();
	for _ in 0..num_workers {
		let rx = rx_work.clone();
		let tx = tx_data.clone();
		let log = sink.log.clone();
		let handle = thread::spawn(move || {
			for entry in rx.iter() {
				if entry.target.len() > 0 {
					// Scrape the target page and add it to the entries.
					let value = match scrape_page(&log, &entry.target) {
						Ok(mut entries) => {
							for it in entries.iter_mut() {
								it.text = entry.text.clone();
							}
							Ok(entries)
						}
						Err(err) => Err(err),
					};
					tx.send(value).unwrap();
				} else {
					// Unlinked entry in the index page? Should not happen, but
					// if it does just use the entry itself.
					tx.send(Ok(vec![entry])).unwrap();
				}
			}
		});
		handles.push(handle);
	}
	drop(rx_work);
	drop(tx_data);

	// Send the index results to the workers.
	for it in index_results {
		tx_work.send(it).unwrap();
	}
	drop(tx_work);

	// Consume the results returned by the workers:

	lazy_static! {
		static ref RE_NATIVE_FROM: Regex = Regex::new(r"(?i)japan").unwrap();
	}

	for it in rx_data {
		match it {
			Ok(entries) => {
				let urls = entries
					.into_iter()
					.sorted_by_key(|it| {
						// Sort entries by natives first, followed by entries
						// that do not specify origin (if any). This is to give
						// precedence for native speakers.
						let native = RE_NATIVE_FROM.is_match(&it.from);
						let empty = it.from.trim().len() == 0;
						(!native, !empty)
					})
					.take(MAX_DETAIL_RESULTS)
					.map(|it| it.mp3.into_iter().take(1))
					.flatten();
				load_audio_list(sink.clone(), urls);
			}
			Err(err) => sink.send_err(err),
		}
	}

	// Wait for all workers to finish. They all should have finished already
	// by the point `rx_data` is dropped.
	for h in handles {
		h.join().unwrap();
	}

	Ok(())
}

#[derive(Debug)]
struct ForvoEntry {
	text:      Vec<String>,
	target:    String,
	from:      String,
	mp3:       Vec<String>,
	ogg:       Vec<String>,
	is_phrase: bool,
}

// DOM format
// ==========
//
// On the main results page:
//
//     <LI>
//         <SPAN id="play_123" class="play" onclick="...">XYZ pronunciation</SPAN>
//         <A class="word" HREF="https://forvo.com/word/XYZ/#ja">XYZ</A>
//     </LI>
//
// The word results page (e.g. `https://forvo.com/word/家_(うち)/#ja`) has the same
// format as above, minus the link.
//
// The onclick handler for the `span.play` is as follows:
//
//     Play(123,'Base64 MP3-A','Base64 OGG-A',false,'Base64 MP3-B','Base64 OGG-B','h')
//
// The `Play` function boils down to:
//
//     function Play(id, mp3_A, ogg_A, is_auto_play, mp3_B, ogg_B, mode) {
//         mp3_A = "https://audio00.forvo.com/mp3/" + base64_decode(mp3_A),
//         ogg_A = "https://audio00.forvo.com/ogg/" + base64_decode(ogg_A);
//         mp3_B = mp3_B && "https://audio00.forvo.com/audios/mp3/" + base64_decode(mp3_B);
//         ogg_B = ogg_B && "https://audio00.forvo.com/audios/ogg/" + base64_decode(ogg_B);
//         createAudioObject(id, mp3_A, ogg_A, is_mobile(), is_auto_play, mp3_B, ogg_B, mode || "l")
//     }
//
//     function createAudioObject(id, mp3_A, ogg_A, mobile, is_auto_play, mp3_B, ogg_B, mode) {
//         let audio = document.createElement("audio");
//         if (mode == "h") {
//             if (mp3_B) add_src(audio, "audio/mp3", mp3_B)
//             if (ogg_B) add_src(audio, "audio/ogg", ogg_B)
//         }
//         if (mp3_A) add_src(audio, "audio/mp3", mp3_A)
//         if (ogg_A) add_src(audio, "audio/ogg", ogg_A)
//         audio.play()
//     }
//
// There is also phrases, which have a slightly different handler, but overall
// the same logic:
//
//     PlayPhrase(123,'Base64 MP3','Base64 OGG')
//
// The audio URLs for phrases are:
//
//     https://audio00.forvo.com/phrases/mp3/
//     https://audio00.forvo.com/phrases/ogg/

fn scrape_page(log: &Logger, url: &str) -> util::Result<Vec<ForvoEntry>> {
	lazy_static! {
		static ref SEL_PLAY: Selector = Selector::parse("li > span.play").unwrap();
		static ref SEL_TARGET: Selector = Selector::parse("a").unwrap();
		static ref SEL_FROM: Selector = Selector::parse("span.from").unwrap();
		static ref RE_PLAY: Regex = Regex::new(r"(?i)^Play(Phrase)?").unwrap();
	}

	let log = log.new(o!("url" => url.to_owned()));
	time!(t_load);

	trace!(log, "loading Forvo results");

	let url = Url::parse(&url)?;
	let client = Client::builder()
		.timeout(Duration::from_millis(DEFAULT_TIMEOUT_MS))
		.build()?;

	let mut response = client.get(url).send()?;
	check_response(&log, &response)?;

	let mut doc = String::new();
	response.read_to_string(&mut doc)?;

	trace!(log, "loaded"; t_load);
	time!(t_parse);

	let doc = Html::parse_document(doc.as_str());

	let mut results = Vec::new();

	'outer: for play in doc.select(&SEL_PLAY) {
		let on_click = match play.value().attr("onclick") {
			Some(v) => v,
			_ => continue,
		};

		let (mp3, ogg, is_phrase) = extract_args(&log, on_click);

		let mut entry = ForvoEntry {
			mp3:       mp3,
			ogg:       ogg,
			is_phrase: is_phrase,
			text:      Vec::new(),
			target:    String::new(),
			from:      String::new(),
		};

		// Look for the `<a class="word" ..>` link on the same parent
		let mut container = play.parent();
		while let Some(current) = container {
			let val = match current.value().as_element() {
				Some(v) => v,
				_ => continue 'outer,
			};
			if val.name() == "li" {
				break;
			}
			container = current.parent();
		}

		match container {
			Some(v) => {
				let container = ElementRef::wrap(v).unwrap();
				if let Some(target) = container.select(&SEL_TARGET).next() {
					entry.text = search_strings(target.text().collect::<String>().trim());
					entry.target = target.value().attr("href").unwrap_or("").to_string();
				} else if let Some(from) = container.select(&SEL_FROM).next() {
					// This is probably the details page, so we grab the `from`
					// information, so we can better filter.
					entry.from = from.text().collect::<String>().trim().to_string();
				}
			}
			_ => continue,
		};

		results.push(entry);
	}

	trace!(log, "parsed {} entries", results.len(); t_parse);

	Ok(results)
}

use data_encoding::BASE64;

/// Extract the URLs for the `(mp3, ogg)` audio files in the `onclick` argument.
fn extract_args(log: &Logger, play: &str) -> (Vec<String>, Vec<String>, bool) {
	lazy_static! {
		static ref RE_PLAY_PHRASE: Regex = Regex::new(r"(?i)^PlayPhrase").unwrap();
		static ref RE_PLAY: Regex = Regex::new(r"(?i)^Play(Phrase)?\((\d+\s*,\s*)?").unwrap();
		static ref RE_RETURN: Regex = Regex::new(r"(?i)\s*;\s*return\s*false(;)?$").unwrap();
		static ref RE_QUOTES: Regex = Regex::new(r#"(?i)(\s*,\s*['"]\w['"])?\s*\)$"#).unwrap();
		static ref RE_BOOLEAN: Regex = Regex::new(r"(?i)\s*,\s*(false|true)\s*").unwrap();
		static ref RE_SPLIT: Regex = Regex::new(r"(?i)\s*,\s*").unwrap();
		static ref RE_QUOTE_CHAR: Regex = Regex::new(r#"["']"#).unwrap();
	};

	let play = play.trim();

	let is_phrase = RE_PLAY_PHRASE.is_match(play);

	let play = RE_PLAY.replace_all(play, "");
	let play = RE_RETURN.replace_all(&play, "");
	let play = RE_QUOTES.replace_all(&play, "");
	let play = RE_BOOLEAN.replace_all(&play, "");

	let mut args = RE_SPLIT
		.split(&play)
		.map(|s| {
			let s = RE_QUOTE_CHAR.replace_all(s, "");
			let s = s.trim();
			if let Ok(data) = BASE64.decode(s.as_bytes()) {
				if let Ok(s) = String::from_utf8(data) {
					s
				} else {
					warn!(log, "forvo::extract_args: failed to encode BASE64 to UTF-8 ({})", s);
					String::new()
				}
			} else {
				warn!(log, "forvo::extract_args: failed to decode BASE64 ({})", s);
				String::new()
			}
		})
		.collect::<Vec<_>>();

	fn rebase(args: &mut Vec<String>, index: usize, base: &str) {
		if index < args.len() {
			args[index] = format!("{}{}", base, args[index]);
		}
	}

	if is_phrase {
		args.truncate(std::cmp::min(args.len(), 2));
		rebase(&mut args, 0, "https://audio00.forvo.com/phrases/mp3/");
		rebase(&mut args, 1, "https://audio00.forvo.com/phrases/ogg/");
	} else {
		args.truncate(std::cmp::min(args.len(), 4));
		rebase(&mut args, 0, "https://audio00.forvo.com/mp3/");
		rebase(&mut args, 1, "https://audio00.forvo.com/ogg/");
		rebase(&mut args, 2, "https://audio00.forvo.com/audios/mp3/");
		rebase(&mut args, 3, "https://audio00.forvo.com/audios/ogg/");
	}

	// The "B" media files (at the end) appear to be encoded in a smaller size,
	// so we prefer those.
	let args = args.into_iter().filter(|x| x.len() > 0).rev().collect::<Vec<_>>();

	lazy_static! {
		static ref RE_MP3: Regex = Regex::new(r"(?i)\.mp3$").unwrap();
		static ref RE_OGG: Regex = Regex::new(r"(?i)\.ogg$").unwrap();
	};

	let mp3 = args.iter().filter(|x| RE_MP3.is_match(x)).cloned().collect::<Vec<_>>();
	let ogg = args.iter().filter(|x| RE_MP3.is_match(x)).cloned().collect::<Vec<_>>();
	(mp3, ogg, is_phrase)
}

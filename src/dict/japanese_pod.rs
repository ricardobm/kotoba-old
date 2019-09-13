use std::error::Error;
use std::io::Read;

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
	pub order: u32,
}

/// Query `japanesepod101.com` dictionary.
pub fn query_dictionary(_args: Args) -> Vec<Entry> {
	panic!()
}

/// Load audio pronunciation from `languagepod101.com`.
pub fn load_audio(kanji: &str, kana: &str) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
	use std::time::Duration;

	use data_encoding::HEXUPPER;
	use regex::Regex;
	use reqwest::header;
	use reqwest::{Client, Url};
	use ring::digest::{Context, SHA256};

	const BLACKLIST_HASH: &str = "AE6398B5A27BC8C0A771DF6C907ADE794BE15518174773C58C7C7DDD17098906";

	lazy_static! {
		static ref MP3_CONTENT_TYPE: Regex = Regex::new(r"mpeg(-?3)?").unwrap();
	}

	let mut url = Url::parse("https://assets.languagepod101.com/dictionary/japanese/audiomp3.php")?;
	url.query_pairs_mut()
		.append_pair("kanji", kanji)
		.append_pair("kana", kana);

	let client = Client::builder().timeout(Duration::from_millis(5000)).build()?;
	let mut response = client.get(url).send()?;
	let status = response.status();
	if status.is_success() {
		if let Some(content_type) = response.headers().get(header::CONTENT_TYPE) {
			if MP3_CONTENT_TYPE.is_match(content_type.to_str()?) {
				Ok(())
			} else {
				Err(format!("response with invalid content type: {:?}", content_type))
			}
		} else {
			Err(format!("response has no content type"))
		}?;

		let mut buffer = Vec::new();
		response.read_to_end(&mut buffer)?;

		if buffer.len() == 0 {
			Err(format!("received empty response"))?;
		}

		let mut context = Context::new(&SHA256);
		context.update(&buffer[..]);
		let digest = context.finish();
		let digest = HEXUPPER.encode(digest.as_ref());
		if digest == BLACKLIST_HASH {
			Ok(None)
		} else {
			Ok(Some(buffer))
		}
	} else {
		if let Some(reason) = status.canonical_reason() {
			Err(format!("request failed with status {} ({})", status, reason).into())
		} else {
			Err(format!("request failed with status {}", status).into())
		}
	}
}

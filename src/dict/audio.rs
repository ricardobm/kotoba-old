use std::error::Error;
use std::io::Read;
use std::time::Duration;

use regex::Regex;
use reqwest::header;
use reqwest::{Client, IntoUrl};

use crate::util::{check_response, sha256};

const DEFAULT_TIMEOUT_MS: u64 = 2500;

/// Audio data and SHA-256.
pub struct AudioData(pub Vec<u8>, pub String);

impl AudioData {
	/// SHA-256 hash for the audio data.
	pub fn hash(&self) -> &str {
		self.1.as_str()
	}
}

/// Load an audio by the given URL.
pub fn load_audio<U: IntoUrl>(url: U) -> Result<AudioData, Box<dyn Error>> {
	lazy_static! {
		static ref MP3_CONTENT_TYPE: Regex = Regex::new(r"mpeg(-?3)?").unwrap();
	}

	let client = Client::builder()
		.timeout(Duration::from_millis(DEFAULT_TIMEOUT_MS))
		.build()?;
	let mut response = client.get(url).send()?;

	match check_response(&response) {
		Ok(_) => {
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

			let digest = sha256(&buffer[..]).unwrap();
			Ok(AudioData(buffer, digest))
		}
		Err(err) => Err(err),
	}
}

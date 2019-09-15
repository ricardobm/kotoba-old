use std::io::Read;
use std::thread;
use std::time::Duration;

use slog::Logger;

use regex::Regex;
use reqwest::header;
use reqwest::{Client, IntoUrl};

use crate::util;
use crate::util::{check_response, sha256};

use crossbeam::channel::unbounded;

const DEFAULT_TIMEOUT_MS: u64 = 2500;

pub type AudioResult = util::Result<AudioData>;

/// Audio data and SHA-256.
pub struct AudioData(pub Vec<u8>, pub String);

impl AudioData {
	/// SHA-256 hash for the audio data.
	pub fn hash(&self) -> &str {
		self.1.as_str()
	}
}

pub fn load_audio_list<T>(log: &Logger, urls: T) -> Vec<AudioResult>
where
	T: IntoIterator<Item = String>,
{
	const MAX_WORKERS: usize = 8;

	let urls: Vec<_> = urls.into_iter().collect();
	let num_workers = std::cmp::min(MAX_WORKERS, urls.len());

	let (tx_work, rx_work) = unbounded::<String>();
	let (tx_data, rx_data) = unbounded::<AudioResult>();

	trace!(
		log,
		"loading {} audio sources using {} workers",
		urls.len(),
		num_workers
	);
	time!(t_load);

	let mut handles = Vec::new();
	for _ in 0..num_workers {
		let rx = rx_work.clone();
		let tx = tx_data.clone();
		let log = log.clone();
		let handle = thread::spawn(move || {
			for url in rx.iter() {
				let result = load_audio(&log, &url);
				tx.send(result).unwrap();
			}
		});
		handles.push(handle);
	}
	drop(rx_work);
	drop(tx_data);

	for url in urls {
		tx_work.send(url).unwrap();
	}
	drop(tx_work);

	let mut out = Vec::new();
	let mut errs = 0;
	for result in rx_data {
		if !result.is_ok() {
			errs += 1;
		}
		out.push(result);
	}

	for h in handles {
		h.join().unwrap();
	}

	trace!(log, "load finished with {} errors", errs; t_load);

	out
}

/// Load an audio by the given URL.
pub fn load_audio<U: IntoUrl>(log: &Logger, url: U) -> AudioResult {
	lazy_static! {
		static ref MP3_CONTENT_TYPE: Regex = Regex::new(r"mpeg(-?3)?").unwrap();
	}

	time!(t_load);

	let client = Client::builder()
		.timeout(Duration::from_millis(DEFAULT_TIMEOUT_MS))
		.build()?;
	let mut response = client.get(url).send()?;

	match check_response(log, &response) {
		Ok(_) => {
			let res = {
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
			};
			let res = if let Err(err) = res {
				warn!(log, "{} when loading {}", err, response.url());
				Err(err)
			} else {
				trace!(log, "{} loaded", response.url(); t_load);
				res
			};
			res
		}
		Err(err) => Err(err),
	}
}

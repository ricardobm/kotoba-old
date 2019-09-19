use std::io::Read;
use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use slog::Logger;

use regex::Regex;
use reqwest::header;
use reqwest::{Client, IntoUrl};

use crate::util;
use crate::util::{check_response, sha256};

use crossbeam::channel::unbounded;

const DEFAULT_TIMEOUT_MS: u64 = 2500;

use crate::audio::{SourceId, AudioDataResult, AudioData};

#[derive(Clone)]
pub struct AudioSink {
	pub log: Logger,
	pub id: SourceId,
	pub sender: std::sync::mpsc::Sender<(SourceId, AudioDataResult)>,
}

impl AudioSink {
	pub fn new(log: Logger, id: SourceId, sender: std::sync::mpsc::Sender<(SourceId, AudioDataResult)>) -> AudioSink {
		AudioSink{ log, id, sender }
	}

	pub fn send(&self, item: AudioDataResult) {
		self.sender.send((self.id.clone(), item)).unwrap();
	}

	pub fn send_data(&self, data: AudioData) {
		self.send(Ok(data))
	}

	pub fn send_err(&self, err: util::Error) {
		self.send(Err(err))
	}
}

pub fn load_audio_list<T>(sink: AudioSink, urls: T)
where
	T: IntoIterator<Item = String>,
{
	const MAX_WORKERS: usize = 8;

	// Spawn a number of workers to load the URLs:

	let urls: Vec<_> = urls.into_iter().collect();
	let num_workers = std::cmp::min(MAX_WORKERS, urls.len());

	// Workers receive URLs to load from here.
	let (tx_work, rx_work) = unbounded::<String>();

	trace!(
		sink.log,
		"loading {} audio sources using {} workers",
		urls.len(),
		num_workers
	);
	time!(t_load);

	let err_count = Arc::new(AtomicUsize::new(0));

	let mut handles = Vec::new();
	for _ in 0..num_workers {
		let rx = rx_work.clone();
		let sink = sink.clone();
		let err_count = err_count.clone();
		let handle = thread::spawn(move || {
			for url in rx.iter() {
				if !load_audio(sink.clone(), &url) {
					err_count.fetch_add(1, Ordering::SeqCst);
				}
			}
		});
		handles.push(handle);
	}
	drop(rx_work);

	// Send URL to the workers
	for url in urls {
		tx_work.send(url).unwrap();
	}
	drop(tx_work);

	for h in handles {
		h.join().unwrap();
	}

	trace!(sink.log, "load finished with {} errors", err_count.load(Ordering::SeqCst); t_load);
}

/// Load an audio by the given URL.
pub fn load_audio<U: IntoUrl>(sink: AudioSink, url: U) -> bool {
	let audio = do_load_audio(&sink.log, url);
	let is_ok = audio.is_ok();
	sink.send(audio);
	is_ok
}

pub fn do_load_audio<U: IntoUrl>(log: &Logger, url: U) -> AudioDataResult {
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
				Ok(AudioData{ data: buffer, hash: digest, cached: false })
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

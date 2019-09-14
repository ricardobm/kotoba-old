use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use itertools::*;
use regex::Regex;

use crate::dict::audio::*;
use crate::dict::forvo;
use crate::dict::japanese_pod;
use crate::dict::jisho;
use crate::util;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum AudioSource {
	Jisho,
	LanguagePod,
	JapanesePod,
	Forvo,
}

impl AudioSource {
	pub fn sub_path(&self) -> &'static str {
		// spell-checker: disable
		match self {
			AudioSource::Jisho => "jisho",
			AudioSource::JapanesePod => "jpod",
			AudioSource::LanguagePod => "langpod",
			AudioSource::Forvo => "forvo",
		}
		// spell-checker: enable
	}
}

impl std::fmt::Display for AudioSource {
	fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> std::fmt::Result {
		write!(f, "{}", self.sub_path())
	}
}

lazy_static! {
	static ref ALL_SOURCES: Vec<AudioSource> = {
		let mut v = vec![
			AudioSource::Jisho,
			AudioSource::JapanesePod,
			AudioSource::LanguagePod,
			AudioSource::Forvo,
		];
		v.sort();
		v
	};
}

/// Provides loading and file-system caching of Japanese pronunciation audio.
#[allow(dead_code)]
pub struct JapaneseService {
	base_path: PathBuf,
}

/// Arguments to load pronunciation audio from [JapaneseService].
pub struct JapaneseQuery {
	/// Main term to lookup.
	pub term: String,

	/// Kana reading for the term to be looked up.
	pub reading: String,

	/// If `true`, this will force loading pronunciation files from the source.
	///
	/// Cached files will still be returned, alongsize any new files that are
	/// found.
	pub force: bool,
}

/// Root result from a pronunciation audio query.
pub struct JapaneseResult {
	/// The original query with normalized strings.
	pub query: JapaneseQuery,

	/// Full hash for the original query string.
	pub query_hash: String,

	/// Root cache directory for this query results. The path is relative to
	/// the [JapaneseService] base path.
	///
	/// Note that this is not guaranteed to actually exist (for example, if the
	/// query returned empty, this will never be created).
	pub cache_path: String,

	/// Items returned by this query. This might be empty in case the query
	/// did not return any results.
	pub items: Vec<JapaneseAudio>,

	/// Collection of errors executing the query. Having errors does not
	/// necessarily prevent the query from having results.
	pub errors: Vec<Box<dyn Error>>,
}

pub enum AudioFile {
	FromFile(fs::DirEntry),
	Buffer(Vec<u8>),
}

/// Audio item from an [AudioResult].
pub struct JapaneseAudio {
	/// Filename (without path information) for the cached audio file.
	pub name: String,

	/// Cached audio file directory's path, relative to the [JapaneseService]
	/// base path.
	pub path: String,

	/// Hash for the content.
	pub hash: String,

	/// Source for this audio.
	pub source: AudioSource,

	/// Data for this entry, if loaded.
	pub data: AudioFile,
}

impl JapaneseAudio {
	/// Return an [io::Read] to access the contents of this item.
	fn buffer(&self) -> Option<&[u8]> {
		match &self.data {
			AudioFile::Buffer(data) => Some(&data[..]),
			_ => None,
		}
	}

	pub fn read(self) -> (String, io::Result<Vec<u8>>) {
		match self.data {
			AudioFile::Buffer(data) => (self.name, Ok(data)),
			AudioFile::FromFile(dir) => (self.name, fs::read(dir.path())),
		}
	}
}

#[allow(dead_code)]
impl JapaneseService {
	/// Returns a new [JapaneseService] instance that loads the cache from the
	/// given base path.
	pub fn new(base_path: &Path) -> JapaneseService {
		JapaneseService {
			base_path: base_path.to_owned(),
		}
	}

	pub fn query(&self, query: JapaneseQuery) -> JapaneseResult {
		lazy_static! {
			static ref ENTRY_RE: Regex = Regex::new(r"^(?P<hash>[0-9a-f]{64})\.mp3$").unwrap();
		}

		let start = std::time::Instant::now();

		let term = normalize_string(&query.term);
		let reading = normalize_string(&query.reading);
		let reading = if reading.len() == 0 {
			crate::kana::to_hiragana(&term)
		} else {
			crate::kana::to_hiragana(&reading)
		};

		let query_hash = format!("{}\n{}", term, reading);
		let query_hash = super::util::sha256(query_hash.as_bytes()).unwrap();

		let cache_path = format!("{}/{}", &query_hash[0..2], &query_hash);

		let mut dir_path = self.base_path.clone();
		for p in cache_path.split('/') {
			dir_path.push(p);
		}

		// Load available entries
		let mut entries_by_src = HashMap::new();
		for src in ALL_SOURCES.iter() {
			let dir_path = dir_path.join(src.sub_path());
			if let Ok(dir) = fs::read_dir(&dir_path) {
				let entries = entries_by_src.entry(*src).or_insert(Vec::new());
				for entry in dir {
					if let Ok(entry) = entry {
						if let Some(name) = entry.file_name().to_str() {
							if let Some(caps) = ENTRY_RE.captures(name) {
								let hash = caps.name("hash").unwrap().as_str();
								entries.push(JapaneseAudio {
									name:   name.to_string(),
									hash:   hash.to_string(),
									path:   format!("{}/{}", cache_path, src.sub_path()),
									source: *src,
									data:   AudioFile::FromFile(entry),
								});
							}
						}
					}
				}
			}
		}

		println!(
			"- Loaded cached entries for {} in {:.3}s",
			entries_by_src.keys().join("/"),
			start.elapsed().as_secs_f64()
		);

		// Load any entry that is not available
		let mut handles = Vec::new();
		let (tx, rx) = mpsc::channel();

		for src in ALL_SOURCES.iter() {
			if entries_by_src.contains_key(src) {
				continue;
			}

			let term = term.clone();
			let reading = reading.clone();
			let tx = tx.clone();
			let src = *src;
			let mut worker = get_worker(src);
			let handle = thread::spawn(move || {
				tx.send((src, worker.load(term, reading))).unwrap();
			});
			handles.push(handle);
		}

		drop(tx);

		let mut out = JapaneseResult {
			query:      JapaneseQuery {
				term:    term,
				reading: reading,
				force:   query.force,
			},
			query_hash: query_hash,
			cache_path: cache_path,
			items:      Vec::new(),
			errors:     Vec::new(),
		};

		for (src, result) in rx {
			let dir_path = dir_path.join(src.sub_path());
			let mut entries = Vec::new();
			let mut success = false;

			match result {
				Ok(all_data) => {
					let err_count = all_data
						.iter()
						.filter(|x| if let Err(_) = x { true } else { false })
						.count();
					println!(
						"- {} loaded {} ({} errors) in {:.3}s",
						src,
						all_data.len(),
						err_count,
						start.elapsed().as_secs_f64()
					);

					let mut is_empty = true;
					for res in all_data {
						is_empty = false;
						match res {
							Ok(AudioData(data, hash)) => {
								let name = format!("{}.mp3", hash);
								entries.push(JapaneseAudio {
									name:   name,
									hash:   hash,
									path:   format!("{}/{}", out.cache_path, src.sub_path()),
									source: src,
									data:   AudioFile::Buffer(data),
								});
								success = true;
							}
							Err(err) => {
								out.errors.push(Box::new(err));
							}
						}
					}
					if is_empty {
						success = true;
					}
				}
				Err(err) => {
					println!("- {} failed after {:.3}s", src, start.elapsed().as_secs_f64());
					out.errors.push(err.into());
				}
			}

			if success {
				match fs::create_dir_all(&dir_path) {
					Ok(_) => {
						for it in entries.iter() {
							if let Err(err) = fs::write(dir_path.join(&it.name), it.buffer().unwrap()) {
								out.errors.push(err.into());
							}
						}
					}
					Err(err) => out.errors.push(err.into()),
				}
				entries_by_src.insert(src, entries);
			}
		}

		for it in handles {
			it.join().unwrap();
		}

		out.items = entries_by_src
			.into_iter()
			.sorted_by(|a, b| a.0.cmp(&b.0))
			.map(|x| x.1.into_iter())
			.flatten()
			.collect();

		println!("- finished in {:.3}s", start.elapsed().as_secs_f64());

		out
	}
}

fn normalize_string(s: &str) -> String {
	use unicode_normalization::UnicodeNormalization;
	let text = s.trim().to_lowercase().nfc().collect::<String>();
	text
}

//
// Audio workers
//

type AudioWorkerResult = util::Result<Vec<AudioResult>>;

trait AudioWorker: Send {
	fn load(&mut self, term: String, reading: String) -> AudioWorkerResult;
}

fn get_worker(src: AudioSource) -> Box<dyn AudioWorker> {
	match src {
		AudioSource::Jisho => Box::new(JishoWorker {}),
		AudioSource::JapanesePod => Box::new(JapanesePodWorker {}),
		AudioSource::LanguagePod => Box::new(LanguagePodWorker {}),
		AudioSource::Forvo => Box::new(ForvoWorker {}),
	}
}

struct JishoWorker {}

impl AudioWorker for JishoWorker {
	fn load(&mut self, term: String, reading: String) -> AudioWorkerResult {
		jisho::load_pronunciations(&term, &reading)
	}
}

struct JapanesePodWorker {}

impl AudioWorker for JapanesePodWorker {
	fn load(&mut self, term: String, reading: String) -> AudioWorkerResult {
		japanese_pod::load_dictionary_pronunciations(&term, &reading)
	}
}

struct LanguagePodWorker {}

impl AudioWorker for LanguagePodWorker {
	fn load(&mut self, term: String, reading: String) -> AudioWorkerResult {
		match japanese_pod::load_pronunciation(&term, &reading)? {
			Some(data) => Ok(vec![Ok(data)]),
			None => Ok(vec![]),
		}
	}
}

struct ForvoWorker {}

impl AudioWorker for ForvoWorker {
	fn load(&mut self, term: String, reading: String) -> AudioWorkerResult {
		forvo::load_pronunciations(&term, &reading)
	}
}

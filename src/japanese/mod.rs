//! Entry point for japanese word and kanji queries.

use std::sync::mpsc::Sender;

use slog::Logger;

pub mod db;
pub mod import;

pub mod dictionary;
pub use self::dictionary::Dictionary;

pub use self::db::search::*;

mod deinflect;
pub use self::deinflect::Inflection;
pub use self::deinflect::deinflect;

//
// Japanese pronunciation audio support
//

use audio::{AudioDataResult, AudioLoader, AudioQuery, AudioSource, SourceId};

pub fn new_audio_loader(base_path: &std::path::Path) -> AudioLoader<JapaneseAudioQuery> {
	AudioLoader::new(
		base_path,
		vec![
			Box::new(JishoSource {}),
			Box::new(JapanesePodSource {}),
			Box::new(LanguagePodSource {}),
			Box::new(ForvoSource {}),
		],
	)
}

pub trait JapaneseAudioSource: AudioSource<JapaneseAudioQuery> {}

/// Query arguments to load Japanese pronunciation audio.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct JapaneseAudioQuery {
	/// The main expression to lookup.
	pub expression: String,
	/// The reading for the expression to lookup.
	pub reading: String,
}

impl AudioQuery for JapaneseAudioQuery {
	fn query_hash(&self) -> String {
		let query_hash = format!("{}\n{}", self.expression, self.reading);
		let query_hash = crate::util::sha256(query_hash.as_bytes()).unwrap();
		query_hash
	}

	fn query_info(&self) -> String {
		format!("{} / {}", self.expression, self.reading)
	}
}

//
// Japanese audio sources
//

mod audio_helper;
mod forvo;
mod japanese_pod;
mod jisho;

use self::audio_helper::AudioSink;

struct JishoSource {}

impl JapaneseAudioSource for JishoSource {}

impl AudioSource<JapaneseAudioQuery> for JishoSource {
	fn copy(&self) -> Box<dyn AudioSource<JapaneseAudioQuery>> {
		Box::new(JishoSource {})
	}

	fn id(&self) -> &'static str {
		"jisho"
	}

	fn name(&self) -> &'static str {
		"Jisho"
	}

	fn load(&self, query: JapaneseAudioQuery, log: Logger, id: SourceId, sink: Sender<(SourceId, AudioDataResult)>) {
		let sink = AudioSink::new(log, id, sink);
		jisho::load_pronunciations(sink, &query.expression, &query.reading);
	}
}

struct JapanesePodSource {}

impl JapaneseAudioSource for JapanesePodSource {}

impl AudioSource<JapaneseAudioQuery> for JapanesePodSource {
	fn copy(&self) -> Box<dyn AudioSource<JapaneseAudioQuery>> {
		Box::new(JapanesePodSource {})
	}

	fn id(&self) -> &'static str {
		// spell-checker: disable
		"jpod"
		// spell-checker: enable
	}

	fn name(&self) -> &'static str {
		"JapanesePod101.com"
	}

	fn load(&self, query: JapaneseAudioQuery, log: Logger, id: SourceId, sink: Sender<(SourceId, AudioDataResult)>) {
		let sink = AudioSink::new(log, id, sink);
		japanese_pod::load_dictionary_pronunciations(sink, &query.expression, &query.reading);
	}
}

struct LanguagePodSource {}

impl JapaneseAudioSource for LanguagePodSource {}

impl AudioSource<JapaneseAudioQuery> for LanguagePodSource {
	fn copy(&self) -> Box<dyn AudioSource<JapaneseAudioQuery>> {
		Box::new(LanguagePodSource {})
	}

	fn id(&self) -> &'static str {
		// spell-checker: disable
		"langpod"
		// spell-checker: enable
	}

	fn name(&self) -> &'static str {
		"LanguagePod101.com"
	}

	fn load(&self, query: JapaneseAudioQuery, log: Logger, id: SourceId, sink: Sender<(SourceId, AudioDataResult)>) {
		let sink = AudioSink::new(log, id, sink);
		japanese_pod::load_pronunciation(sink, &query.expression, &query.reading);
	}
}

struct ForvoSource {}

impl JapaneseAudioSource for ForvoSource {}

impl AudioSource<JapaneseAudioQuery> for ForvoSource {
	fn copy(&self) -> Box<dyn AudioSource<JapaneseAudioQuery>> {
		Box::new(ForvoSource {})
	}

	fn id(&self) -> &'static str {
		// spell-checker: disable
		"forvo"
		// spell-checker: enable
	}

	fn name(&self) -> &'static str {
		"Forvo"
	}

	fn load(&self, query: JapaneseAudioQuery, log: Logger, id: SourceId, sink: Sender<(SourceId, AudioDataResult)>) {
		let sink = AudioSink::new(log, id, sink);
		forvo::load_pronunciations(sink, &query.expression, &query.reading);
	}
}

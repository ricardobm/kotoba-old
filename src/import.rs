//! Support for importing data from Yomichan style dictionaries.

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde_json;

/// Dictionary data imported from a Yomichan internal format.
#[derive(Deserialize)]
pub struct Dict {
	/// Dictionary name.
	pub title: String,

	/// Dictionary format (expected `3`).
	pub format: u32,

	/// Dictionary revision tag.
	pub revision: String,

	/// Unused
	#[serde(default)]
	pub sequenced: bool,

	/// Source path for the dictionary.
	#[serde(skip)]
	pub path: PathBuf,

	/// List of imported terms.
	#[serde(skip)]
	pub terms: Vec<Term>,

	/// List of imported kanjis.
	#[serde(skip)]
	pub kanjis: Vec<Kanji>,

	/// Definition of tags used by the dictionary terms/kanjis.
	#[serde(skip)]
	pub tags: Vec<Tag>,

	/// Frequency metadata for terms.
	#[serde(skip)]
	pub meta_terms: Vec<Meta>,

	/// Frequency metadata for kanjis.
	#[serde(skip)]
	pub meta_kanjis: Vec<Meta>,
}

/// Dictionary entry for a term.
///
/// Each entry contains a single definition for the term given by `expression`.
/// The definition itself consists of one or more `glossary` items.
pub struct Term {
	/// Term expression.
	pub expression: String,

	/// Kana reading for this term.
	pub reading: String,

	/// Tags for the term definitions.
	pub definition_tags: Vec<String>,

	/// Rules that affect the entry inflections. Those are also tags.
	///
	/// One of `adj-i`, `v1`, `v5`, `vk`, `vs`.
	///
	/// - `adj-i` adjective (keiyoushi)
	/// - `v1`    Ichidan verb
	/// - `v5`    Godan verb
	/// - `vk`    Kuru verb - special class (e.g. `いって来る`, `來る`)
	/// - `vs`    noun or participle which takes the aux. verb suru
	pub rules: Vec<String>,

	/// Score for this entry. Higher values have precedence.
	pub score: i32,

	/// Definition for this entry.
	pub glossary: Vec<String>,

	/// Sequence number for this entry in the dictionary.
	pub sequence: i32,

	/// Tags for the main term.
	pub term_tags: Vec<String>,
}

/// Dictionary entry for a kanji.
pub struct Kanji {
	/// Kanji character.
	pub character: char,

	/// Onyomi (chinese) readings for the Kanji.
	pub onyomi: Vec<String>,

	/// Kunyomi (japanese) readings for the Kanji.
	pub kunyomi: Vec<String>,

	/// Tags for the Kanji.
	pub tags: Vec<String>,

	/// Meanings for the kanji.
	pub meanings: Vec<String>,

	/// Additional kanji information. The keys in `stats` are further detailed
	/// by the dictionary tags.
	pub stats: HashMap<String, String>,
}

/// Tag for an `Kanji` or `Term`. For kanji dictionary.
///
/// For a `Kanji`, this is also used to describe the `stats` keys.
pub struct Tag {
	/// Name to reference this tag.
	pub name: String,

	/// Category for this tag. This can be used to group related tags.
	pub category: String,

	/// Sort order for this tag (less is higher). This has higher priority than
	/// the name.
	pub order: i32,

	/// Description for this tag.
	pub notes: String,

	/// Unused.
	pub score: i32,
}

/// Frequency metadata for kanjis or terms.
pub struct Meta {
	/// Kanji or term.
	pub expression: String,

	/// Always `"freq"`.
	pub mode: String,

	/// Metadata value.
	pub data: i32,
}

enum DataKind {
	Term,
	Kanji,
	Tag,
	KanjiMeta,
	TermMeta,
}

pub fn import_from(import_path: &Path) -> io::Result<Vec<Dict>> {
	let mut results: Vec<Dict> = Vec::new();
	for entry in fs::read_dir(import_path)? {
		let entry = entry?;
		let fullpath = entry.path();
		let is_file = entry.file_type()?.is_file();
		if is_file {
			if let Some(ext) = fullpath.extension() {
				if ext == "zip" {
					results.push(import_from_zip(&fullpath)?);
				}
			}
		} else {
			let mut index_path = PathBuf::from(fullpath.clone());
			index_path.push("index.json");
			if let Ok(md) = fs::metadata(index_path) {
				if md.is_file() {
					results.push(import_from_dir(&fullpath)?);
				}
			}
		}
	}
	Ok(results)
}

fn import_from_zip(_zip_path: &Path) -> io::Result<Dict> {
	panic!("NIE")
}

fn import_from_dir(dir_path: &Path) -> io::Result<Dict> {
	let mut index_path = dir_path.to_path_buf();
	index_path.push("index.json");

	let index_file = BufReader::new(File::open(index_path)?);

	let mut dict: Dict = serde_json::from_reader(index_file)?;
	dict.path = PathBuf::from(dir_path);

	for entry in fs::read_dir(dir_path)? {
		let entry = entry?;

		// Basic checks:

		if entry.file_name() == "index.json" {
			continue; // we already processed the index.json file
		}

		let entry_path = entry.path();
		if let Some(ext) = entry_path.extension() {
			if ext != "json" {
				continue; // not a JSON file
			}
		} else {
			continue; // no extension
		}

		if !entry.metadata()?.is_file() {
			continue;
		}

		let kind = get_kind(&entry.file_name().to_string_lossy());
		if let Some(kind) = kind {
			let entry_file = BufReader::new(File::open(entry_path)?);
			match kind {
				DataKind::Term => {
					#[derive(Deserialize)]
					struct TermRow(
						String,      // expression
						String,      // reading
						String,      // definition tags (CSV)
						String,      // rules (CSV)
						i32,         // score
						Vec<String>, // glossary
						i32,         // sequence
						String,      // term tags (CSV)
					);
					let rows: Vec<TermRow> = serde_json::from_reader(entry_file)?;
					for it in rows {
						dict.terms.push(Term {
							expression:      it.0,
							reading:         it.1,
							definition_tags: csv(&it.2),
							rules:           csv(&it.3),
							score:           it.4,
							glossary:        it.5,
							sequence:        it.6,
							term_tags:       csv(&it.7),
						});
					}
				}
				DataKind::Kanji => {
					#[derive(Deserialize)]
					struct KanjiRow(
						char,                    // character
						String,                  // onyomi (CSV)
						String,                  // kunyomi (CSV)
						String,                  // tags (CSV)
						Vec<String>,             // meanings
						HashMap<String, String>, // stats
					);
					let rows: Vec<KanjiRow> = serde_json::from_reader(entry_file)?;
					for it in rows {
						dict.kanjis.push(Kanji {
							character: it.0,
							onyomi:    csv(&it.1),
							kunyomi:   csv(&it.2),
							tags:      csv(&it.3),
							meanings:  it.4,
							stats:     it.5,
						});
					}
				}
				DataKind::Tag => {
					#[derive(Deserialize)]
					struct TagRow(
						String, // name
						String, // category
						i32,    // order
						String, // notes
						i32,    // score
					);
					let rows: Vec<TagRow> = serde_json::from_reader(entry_file)?;
					for it in rows {
						dict.tags.push(Tag {
							name:     it.0,
							category: it.1,
							order:    it.2,
							notes:    it.3,
							score:    it.4,
						});
					}
				}
				DataKind::KanjiMeta => {
					dict.meta_kanjis = read_meta(entry_file)?;
				}
				DataKind::TermMeta => {
					dict.meta_terms = read_meta(entry_file)?;
				}
			}
		}
	}

	Ok(dict)
}

fn read_meta<R: io::Read>(input: BufReader<R>) -> io::Result<Vec<Meta>> {
	#[derive(Deserialize)]
	struct MetaRow(
		String, // expression
		String, // mode
		i32,    // data
	);
	let rows: Vec<MetaRow> = serde_json::from_reader(input)?;
	let mut result: Vec<Meta> = Vec::new();
	for it in rows {
		result.push(Meta {
			expression: it.0,
			mode:       it.1,
			data:       it.2,
		});
	}
	Ok(result)
}

fn csv(ls: &str) -> Vec<String> {
	if ls.len() == 0 {
		Vec::new()
	} else {
		ls.split(",").map(|s| String::from(s)).collect()
	}
}

fn get_kind(file_name: &str) -> Option<DataKind> {
	lazy_static! {
		static ref RE: Regex = Regex::new(r"(_bank(_\d+)?)?\.json$").unwrap();
	}
	match RE.replace_all(file_name, "").to_lowercase().as_str() {
		"term" => Some(DataKind::Term),
		"kanji" => Some(DataKind::Kanji),
		"tag" => Some(DataKind::Tag),
		"kanji_meta" => Some(DataKind::KanjiMeta),
		"term_meta" => Some(DataKind::TermMeta),
		_ => None,
	}
}

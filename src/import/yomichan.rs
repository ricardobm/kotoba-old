//! Support for importing data from Yomichan style dictionaries.

use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde_json;
use zip::ZipArchive;

use super::super::db;

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

	/// List of imported kanji.
	#[serde(skip)]
	pub kanji: Vec<Kanji>,

	/// Definition of tags used by the dictionary terms/kanji.
	#[serde(skip)]
	pub tags: Vec<Tag>,

	/// Frequency metadata for terms.
	#[serde(skip)]
	pub meta_terms: Vec<Meta>,

	/// Frequency metadata for kanji.
	#[serde(skip)]
	pub meta_kanji: Vec<Meta>,
}

impl Dict {
	#[allow(dead_code)]
	pub fn append_to<'a>(&self, db: &mut db::Root) {
		// Add ourselves as a source
		let source_index = db.add_source(db::SourceRow {
			name:     self.title.clone(),
			revision: self.revision.clone(),
		});

		// Add all registered tags
		let mut tag_map = HashMap::new();
		for it in &self.tags {
			let tag_id = db.add_tag(db::TagRow {
				name:        it.name.clone(),
				category:    it.category.clone(),
				description: it.notes.clone(),
				order:       it.order,
				source:      source_index,
			});
			tag_map.insert(Cow::from(&it.name), tag_id);
		}

		// Add terms
		for it in self.terms.iter() {
			let tags = it.term_tags.iter().chain(it.rules.iter());
			let term = db::TermRow {
				expression: it.expression.clone(),
				reading:    it.reading.clone(),
				romaji:     String::new(),
				definition: vec![db::DefinitionRow {
					text: it.glossary.clone(),
					info: Vec::new(),
					tags: add_tags(db, source_index, &mut tag_map, &it.definition_tags),
					link: Vec::new(),
				}],
				source:     vec![source_index],
				forms:      Vec::new(),
				tags:       add_tags(db, source_index, &mut tag_map, tags),
				frequency:  None,
				score:      it.score,
			};
			db.add_term(term);
		}

		// Add kanji
		for it in &self.kanji {
			let mut kanji = db::KanjiRow {
				character: it.character,
				onyomi:    it.onyomi.clone(),
				kunyomi:   it.kunyomi.clone(),
				tags:      add_tags(db, source_index, &mut tag_map, &it.tags),
				meanings:  it.meanings.clone(),
				stats:     HashMap::new(),
				frequency: None,
			};
			add_tags(db, source_index, &mut tag_map, it.stats.keys());
			for (key, value) in &it.stats {
				let tag_id = tag_map.get(&Cow::from(key)).unwrap();
				kanji.stats.insert(*tag_id, value.clone());
			}
			db.add_kanji(kanji);
		}

		// Add meta rows for kanji
		for it in &self.meta_kanji {
			let meta = db::MetaRow {
				expression: it.expression.clone(),
				value:      it.data,
			};
			db.add_meta_kanji(meta);
		}

		// Add meta rows for terms
		for it in &self.meta_terms {
			let meta = db::MetaRow {
				expression: it.expression.clone(),
				value:      it.data,
			};
			db.add_meta_terms(meta);
		}
	}
}

/// Adds any missing `tags` to the database and returns a HashSet with all the
/// given tags' IDs.
fn add_tags<'a, L, S>(
	db: &mut db::Root,
	source_index: db::SourceIndex,
	tag_map: &mut HashMap<Cow<'a, str>, db::TagId>,
	tags: L,
) -> HashSet<db::TagId>
where
	L: IntoIterator<Item = S>,
	S: Into<Cow<'a, str>>,
{
	let mut out = HashSet::new();
	for tag in tags.into_iter() {
		let key = tag.into();
		match tag_map.get(&key) {
			Some(&tag_id) => {
				out.insert(tag_id);
			}
			None => {
				let tag_id = db.add_tag(db::TagRow {
					name:        String::from(key.as_ref()),
					category:    String::new(),
					description: String::new(),
					order:       0,
					source:      source_index,
				});
				tag_map.insert(key, tag_id);
				out.insert(tag_id);
			}
		}
	}
	out
}

impl fmt::Display for Dict {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"=> {} [{}({})] -- {}\n",
			self.title,
			self.revision,
			self.format,
			self.path.to_string_lossy()
		)?;
		write!(f, " - Terms:  {}\n", self.terms.len())?;
		write!(f, " - Kanji: {}\n", self.kanji.len())?;
		write!(f, " - Tags:   {}\n", self.tags.len())?;
		write!(
			f,
			" - Meta:   {} / {} (terms / kanji)",
			self.meta_terms.len(),
			self.meta_kanji.len()
		)?;
		Ok(())
	}
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

impl fmt::Display for Term {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "-> {}", self.expression)?;
		if self.reading.len() > 0 {
			write!(f, " 「{}」", self.reading)?;
		}
		write!(f, " -- {}/{}", self.sequence, self.score)?;
		if self.term_tags.len() > 0 {
			write!(f, "  {}", self.term_tags.join(", "))?;
		}
		writeln!(f)?;

		let tags = self.definition_tags.len();
		let rules = self.rules.len();
		if tags > 0 || rules > 0 {
			write!(f, "   [")?;
			if tags > 0 {
				write!(f, "tags: {}", self.definition_tags.join(", "))?;
			}
			if rules > 0 {
				if tags > 0 {
					write!(f, " / ")?;
				}
				write!(f, "rules: {}", self.rules.join(", "))?;
			}
			write!(f, "]\n")?;
		}

		write!(f, "   {}", self.glossary.join("; "))?;
		Ok(())
	}
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

impl fmt::Display for Kanji {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "-> {}", self.character)?;
		let on = self.onyomi.len();
		let kun = self.kunyomi.len();
		if on > 0 || kun > 0 {
			write!(f, " 「")?;
			if on > 0 {
				write!(f, "ON: {}", self.onyomi.join("  "))?;
			}
			if kun > 0 {
				if on > 0 {
					write!(f, " / ")?;
				}
				write!(f, "KUN: {}", self.kunyomi.join("  "))?;
			}
			write!(f, " 」")?;
		}
		writeln!(f)?;
		if self.tags.len() > 0 {
			write!(f, "   [{}]\n", self.tags.join(", "))?;
		}
		write!(f, "   {}", self.meanings.join("; "))?;
		if self.stats.len() > 0 {
			let mut pairs: Vec<_> = self.stats.iter().collect();
			pairs.sort();
			let pairs: Vec<_> = pairs
				.into_iter()
				.map(|(key, val)| format!("{}: {}", key, val))
				.collect();
			let mut counter = 0;
			for it in pairs {
				if counter % 8 == 0 {
					write!(f, "\n   : ")?;
				} else {
					write!(f, ", ")?;
				}
				write!(f, "{}", it)?;
				counter += 1;
			}
		}
		Ok(())
	}
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

impl fmt::Display for Tag {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{} ({}): {} -- {}/{}",
			self.name, self.category, self.notes, self.order, self.score
		)
	}
}

/// Frequency metadata for kanji or terms.
pub struct Meta {
	/// Kanji or term.
	pub expression: String,

	/// Always `"freq"`.
	pub mode: String,

	/// Metadata value.
	pub data: u64,
}

impl fmt::Display for Meta {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{} = {} ({})", self.expression, self.data, self.mode)
	}
}

enum DataKind {
	Term,
	Kanji,
	Tag,
	KanjiMeta,
	TermMeta,
}

#[allow(dead_code)]
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

fn import_from_zip(zip_path: &Path) -> io::Result<Dict> {
	let file = File::open(zip_path)?;
	let mut archive = ZipArchive::new(file)?;

	let index_file = archive.by_name("index.json")?;

	let mut dict: Dict = serde_json::from_reader(index_file)?;
	dict.path = PathBuf::from(zip_path);

	for i in 0..archive.len() {
		let file = archive.by_index(i)?;
		if !file.is_file() {
			continue;
		}

		let path = file.sanitized_name();
		let name = path.to_string_lossy();

		// Basic checks:

		if name == "index.json" {
			continue; // we already processed the index.json file
		}

		if let Some(ext) = path.extension() {
			if ext != "json" {
				continue; // not a JSON file
			}
		} else {
			continue; // no extension
		}

		import_entry(&mut dict, &name, || Ok(file))?;
	}

	Ok(dict)
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

		let filename = entry.file_name();
		let filename = filename.to_string_lossy();
		import_entry(&mut dict, &filename, || Ok(BufReader::new(File::open(entry_path)?)))?;
	}

	Ok(dict)
}

fn import_entry<F, R>(dict: &mut Dict, filename: &str, open: F) -> io::Result<()>
where
	F: FnOnce() -> io::Result<R>,
	R: io::Read,
{
	let kind = get_kind(filename);
	if let Some(kind) = kind {
		let entry_file = open()?;
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
					dict.kanji.push(Kanji {
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
				for it in read_meta(entry_file)? {
					dict.meta_kanji.push(it);
				}
			}
			DataKind::TermMeta => {
				for it in read_meta(entry_file)? {
					dict.meta_terms.push(it);
				}
			}
		}
	}

	Ok(())
}

fn read_meta<R: io::Read>(input: R) -> io::Result<Vec<Meta>> {
	#[derive(Deserialize)]
	struct MetaRow(
		String, // expression
		String, // mode
		u64,    // data
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
		ls.split(' ').map(|s| String::from(s)).collect()
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

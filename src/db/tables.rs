use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io;
use std::path::Path;

/// Main serialization structure for the dictionary database.
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Root {
	pub kanjis:  Vec<KanjiRow>,
	pub terms:   Vec<TermRow>,
	pub tags:    Vec<TagRow>,
	pub sources: Vec<SourceRow>,
	pub meta:    Vec<MetaRow>,

	tag_keys: HashSet<String>,
}

impl Root {
	pub fn new() -> Root {
		Root {
			kanjis:  Vec::new(),
			terms:   Vec::new(),
			tags:    Vec::new(),
			sources: Vec::new(),
			meta:    Vec::new(),

			tag_keys: HashSet::new(),
		}
	}

	/// Attempt to load the database from the given path.
	///
	/// - Returns `Ok(Some(database))` if successful.
	/// - If the path does not exists, returns `Ok(None)`.
	/// - If the path exists, but there was an error loading, returns `Err(_)`.
	pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Option<Root>> {
		let path = path.as_ref();
		if !path.exists() {
			Ok(None)
		} else {
			let db_file = File::open(path)?;
			let db_file = io::BufReader::new(db_file);
			let db: Root = match bincode::deserialize_from(db_file) {
				Ok(val) => val,
				Err(err) => {
					return io::Result::Err(io::Error::new(io::ErrorKind::Other, err));
				}
			};
			Ok(Some(db))
		}
	}

	/// Save the database to the given path.
	pub fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
		let fs = File::create(path)?;
		let fs = io::BufWriter::new(fs);
		if let Err(err) = bincode::serialize_into(fs, self) {
			return io::Result::Err(io::Error::new(io::ErrorKind::Other, err));
		}

		Ok(())
	}

	/// Add a [TagRow] to the database, returning the new [TagId].
	///
	/// Note that this will automatically generate a unique [TagRow::key]
	/// based on the existing one (or the tag name if it is empty) by appending
	/// a counter to it.
	pub fn add_tag(&mut self, mut tag: TagRow) -> TagId {
		// Setup a unique key for the tag:

		if tag.key.len() == 0 {
			tag.key = tag.name.clone();
		}
		if tag.key.len() > 0 {
			tag.key.push('-');
		}

		let mut counter = 1;
		loop {
			let new_key = format!("{}{}", tag.key, counter);
			if !self.tag_keys.contains(&new_key) {
				tag.key = new_key.clone();
				self.tag_keys.insert(new_key);
				break;
			} else {
				counter += 1;
			}
		}

		self.tags.push(tag);
		TagId(self.tags.len())
	}

	/// Add a [SourceRow] to the database, returning the new [SourceId].
	pub fn add_source(&mut self, source: SourceRow) -> SourceId {
		self.sources.push(source);
		SourceId(self.sources.len())
	}
}

/// Entry for a kanji in the dictionary.
#[derive(Serialize, Deserialize, Debug)]
pub struct KanjiRow {
	/// Kanji character.
	pub character: char,

	/// Onyomi (chinese) readings for the Kanji.
	pub onyomi: Vec<String>,

	/// Kunyomi (japanese) readings for the Kanji.
	pub kunyomi: Vec<String>,

	/// ID of the tags that apply to this kanji.
	pub tags: HashSet<TagId>,

	/// English meanings for the kanji.
	pub meanings: Vec<String>,

	/// Additional kanji information. The key meanings are detailed as tags.
	pub stats: HashMap<TagId, String>,

	/// Frequency information for this Kanji, when available.
	pub frequency: Option<u64>,
}

/// Main entry for a word in the dictionary.
#[derive(Serialize, Deserialize, Debug)]
pub struct TermRow {
	/// Main expression.
	pub expression: String,

	/// Kana reading for the main expression.
	pub reading: String,

	/// Definitions for this term.
	pub definition: Vec<DefinitionRow>,

	/// Description of the origin for this entry.
	pub source: SourceId,

	/// Additional forms for the term, if any.
	pub forms: Vec<FormRow>,

	/// ID of the tags that apply to this term.
	pub tags: HashSet<TagId>,

	/// Frequency information for this term, when available.
	pub frequency: Option<u64>,

	/// Numeric score of this entry to be used as a tie breaker for sorting.
	///
	/// Higher values should appear first. Note that using this with entries
	/// from different sources is meaningless.
	pub score: i32,
}

/// English definition for a [TermRow].
#[derive(Serialize, Deserialize, Debug)]
pub struct DefinitionRow {
	/// Definition text entries.
	pub text: Vec<String>,

	/// Additional information to append after this entry text.
	pub info: Vec<String>,

	/// ID of the tags that apply to this definition.
	pub tags: HashSet<TagId>,

	/// Resources linked to this definition.
	pub link: Vec<LinkRow>,
}

/// Additional forms for a [TermRow].
#[derive(Serialize, Deserialize, Debug)]
pub struct FormRow {
	/// Term expression.
	pub expression: String,

	/// Kana reading for this term.
	pub reading: String,
}

/// Linked resource in the form.
#[derive(Serialize, Deserialize, Debug)]
pub struct LinkRow {
	pub uri:   String,
	pub title: String,
}

/// Index for a [SourceRow] in a [Root].
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct SourceId(usize);

/// Available sources for [KanjiRow] and [TermRow].
#[derive(Serialize, Deserialize, Debug)]
pub struct SourceRow {
	/// Source name.
	pub name: String,

	/// Source revision version.
	pub revision: String,
}

/// Frequency metadata for kanjis or terms.
#[derive(Serialize, Deserialize, Debug)]
pub struct MetaRow {
	/// Term or kanji.
	pub expression: String,

	/// Frequency value.
	pub value: u64,

	/// True when the frequency is for a kanji.
	pub kanji: bool,
}

/// Index for a [TagRow] in a [Root].
#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct TagId(usize);

/// Tag for an [KanjiRow] or [TermRow].
///
/// For kanjis, this is also used to describe the `stats` keys.
#[derive(Serialize, Deserialize, Debug)]
pub struct TagRow {
	/// Unique string key for this tag.
	pub key: String,

	/// Short display name for this tag.
	pub name: String,

	/// Category name for this tag.
	pub category: String,

	/// Longer display description for this tag.
	pub description: String,

	/// Order value for this tag (lesser values sorted first).
	pub order: i32,
}

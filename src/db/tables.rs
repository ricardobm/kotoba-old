use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io;
use std::path::Path;

use itertools::*;

use super::index::Index;

use crate::kana::{to_hiragana, to_romaji};

/// Main serialization structure for the dictionary database.
#[derive(Serialize, Deserialize)]
pub struct Root {
	pub kanji:      Vec<KanjiRow>,
	pub terms:      Vec<TermRow>,
	pub tags:       Vec<TagRow>,
	pub sources:    Vec<SourceRow>,
	pub meta_kanji: Vec<MetaRow>,
	pub meta_terms: Vec<MetaRow>,

	#[serde(skip)]
	from: String,

	#[serde(skip)]
	pub(super) index: Index,
}

trait DbDisplay {
	fn fmt<W: std::io::Write>(&self, root: &Root, f: &mut W) -> io::Result<()>;
}

impl Root {
	pub fn new() -> Root {
		Root {
			kanji:      Vec::new(),
			terms:      Vec::new(),
			tags:       Vec::new(),
			sources:    Vec::new(),
			meta_kanji: Vec::new(),
			meta_terms: Vec::new(),
			from:       String::from("new"),
			index:      Index::default(),
		}
	}

	/// Attempt to load the database from the given path.
	///
	/// - Returns `Ok(Some(database))` if successful.
	/// - If the path does not exists, returns `Ok(None)`.
	/// - If the path exists, but there was an error loading, returns `Err(_)`.
	pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Option<Root>> {
		let path = path.as_ref();
		let from = path.to_string_lossy();
		if !path.exists() {
			Ok(None)
		} else {
			let db_file = File::open(path)?;
			let db_file = io::BufReader::new(db_file);
			let mut db: Root = match bincode::deserialize_from(db_file) {
				Ok(val) => val,
				Err(err) => {
					return io::Result::Err(io::Error::new(io::ErrorKind::Other, err));
				}
			};
			db.from = String::from(from);
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

	/// Updates the internal indexes of the database.
	pub fn update_index(&mut self) {
		self.index.clear();

		// Map all kanji in the database
		for (index, kanji) in self.kanji.iter().enumerate() {
			self.index.map_kanji(kanji.character, index);
		}

		let all_terms = self.terms.iter().enumerate().map(|(index, term)| {
			let keys = vec![&term.expression, &term.reading].into_iter().chain(
				term.forms
					.iter()
					.map(|x| vec![&x.expression, &x.reading].into_iter())
					.flatten(),
			);
			(index, keys)
		});
		self.index.map_term_keywords(all_terms);
		self.index.dump_info();
	}

	/// Add a [TagRow] to the database, returning the new [TagId].
	///
	/// Note that this will automatically generate a unique [TagRow::key]
	/// based on the existing one (or the tag name if it is empty) by appending
	/// a counter to it.
	pub fn add_tag(&mut self, tag: TagRow) -> TagId {
		self.tags.push(tag);
		TagId(self.tags.len() - 1)
	}

	/// Get a copy of the tag with the given ID.
	pub fn get_tag(&self, id: TagId) -> TagRow {
		let TagId(index) = id;
		self.tags[index].clone()
	}

	/// Add a [KanjiRow] to the database.
	pub fn add_kanji(&mut self, kanji: KanjiRow) {
		self.kanji.push(kanji);
	}

	/// Add a [TermRow] to the database.
	pub fn add_term(&mut self, mut term: TermRow) {
		// Complete the reading for all forms. In general, any term with no
		// reading is just kana, so we can translate it to hiragana.
		if term.reading.len() == 0 {
			term.reading = to_hiragana(&term.expression);
		}
		term.romaji = to_romaji(&term.reading);

		for it in term.forms.iter_mut() {
			if it.reading.len() == 0 {
				it.reading = to_hiragana(&it.expression);
			}
			it.romaji = to_romaji(&it.reading);
		}

		self.terms.push(term);
	}

	/// Add a [MetaRow] for kanji to the database.
	pub fn add_meta_kanji(&mut self, meta: MetaRow) {
		self.meta_kanji.push(meta);
	}

	/// Add a [MetaRow] for terms to the database.
	pub fn add_meta_terms(&mut self, meta: MetaRow) {
		self.meta_terms.push(meta);
	}

	/// Add a [SourceRow] to the database, returning the new [SourceId].
	pub fn add_source(&mut self, source: SourceRow) -> SourceId {
		self.sources.push(source);
		SourceId(self.sources.len() - 1)
	}

	/// Dump a sample of the data.
	#[allow(dead_code)]
	pub fn dump<W>(&self, sample_n: usize, f: &mut W) -> io::Result<()>
	where
		W: std::io::Write,
	{
		write!(f, "# Database ({}) #", self.from)?;

		if sample_n > 0 {
			use rand::prelude::SliceRandom;

			let mut rng = rand::thread_rng();

			if self.kanji.len() > 0 {
				let mut kanji = self.kanji.iter().collect::<Vec<_>>();
				kanji.sort_by(|a, b| (b.frequency, b.character).cmp(&(a.frequency, a.character)));
				kanji.truncate(2500);

				write!(f, "\n\n## Kanji")?;
				kanji.as_mut_slice().shuffle(&mut rng);
				for (i, it) in kanji.into_iter().take(sample_n).enumerate() {
					write!(f, "\n\n=> {:02} - ", i + 1)?;
					it.fmt(self, f)?;
				}
			}

			if self.terms.len() > 0 {
				let mut terms = self.terms.iter().collect::<Vec<_>>();
				terms.sort_by(|a, b| (b.frequency, &b.expression).cmp(&(a.frequency, &a.expression)));
				terms.truncate(5000);

				write!(f, "\n\n## Terms")?;
				terms.as_mut_slice().shuffle(&mut rng);
				for (i, it) in terms.into_iter().take(sample_n).enumerate() {
					write!(f, "\n\n=> {:02} - ", i + 1)?;
					it.fmt(self, f)?;
				}
			}

			if self.meta_kanji.len() > 0 {
				let mut meta = self.meta_kanji.iter().collect::<Vec<_>>();
				write!(f, "\n\n## Meta (kanji)\n")?;
				meta.as_mut_slice().shuffle(&mut rng);
				for (i, it) in meta.into_iter().take(sample_n).enumerate() {
					write!(f, "\n=> {:02} - {}", i + 1, it)?;
				}
			}

			if self.meta_terms.len() > 0 {
				let mut meta = self.meta_terms.iter().collect::<Vec<_>>();
				write!(f, "\n\n## Meta (terms)\n")?;
				meta.as_mut_slice().shuffle(&mut rng);
				for (i, it) in meta.into_iter().take(sample_n).enumerate() {
					write!(f, "\n=> {:02} - {}", i + 1, it)?;
				}
			}

			if self.tags.len() > 0 {
				let mut tags = self.tags.iter().collect::<Vec<_>>();
				write!(f, "\n\n## Tags\n")?;
				tags.sort_by(|a, b| {
					let key_a = (a.name.to_lowercase(), a.source);
					let key_b = (b.name.to_lowercase(), b.source);
					key_a.cmp(&key_b)
				});
				for it in tags {
					write!(f, "\n   | {}", it)?;
				}
			}
		}

		if self.sources.len() > 0 {
			write!(f, "\n\n## Sources ##\n")?;
			for (i, it) in self.sources.iter().enumerate() {
				write!(f, "\n   {}. {}", i + 1, it)?;
			}
		}

		write!(f, "\n\n## Summary ##\n")?;
		if self.sources.len() > 0 {
			write!(f, "\n- {} sources", self.sources.len())?;
		}
		if self.kanji.len() > 0 {
			write!(f, "\n- {} kanji", self.kanji.len())?;
		}
		if self.terms.len() > 0 {
			write!(f, "\n- {} terms", self.terms.len())?;
		}
		if self.tags.len() > 0 {
			write!(f, "\n- {} tags", self.tags.len())?;
		}
		if self.meta_kanji.len() > 0 {
			write!(f, "\n- {} meta (kanji)", self.meta_kanji.len())?;
		}
		if self.meta_terms.len() > 0 {
			write!(f, "\n- {} meta (terms)", self.meta_terms.len())?;
		}

		Ok(())
	}

	/// Update frequency information for terms and kanji.
	///
	/// Returns the updated count for `(kanji, terms)`.
	pub fn update_frequency(&mut self) -> (usize, usize) {
		let mut kanji_count = 0;
		let mut terms_count = 0;

		//
		// Update kanji:
		//

		let mut kanji_map = HashMap::new();
		for it in &self.meta_kanji {
			if let Some(chr) = it.expression.chars().next() {
				kanji_map.insert(chr, it.value);
				kanji_count += 1;
			}
		}

		for it in self.kanji.iter_mut() {
			if let Some(value) = kanji_map.get(&it.character) {
				it.frequency = Some(*value);
			}
		}

		//
		// Update terms:
		//

		let mut terms_map = HashMap::new();
		for it in &self.meta_terms {
			terms_map.insert(it.expression.as_str(), it.value);
		}

		for it in self.terms.iter_mut() {
			if let Some(value) = terms_map.get(it.expression.as_str()) {
				it.frequency = Some(*value);
				terms_count += 1;
			}
		}

		(kanji_count, terms_count)
	}
}

/// Entry for a kanji in the dictionary.
#[derive(Serialize, Deserialize, Debug, Clone)]
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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TermRow {
	/// Main expression.
	pub expression: String,

	/// Kana reading for the main expression.
	pub reading: String,

	/// Kana reading as romaji.
	pub romaji: String,

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
#[derive(Serialize, Deserialize, Debug, Clone)]
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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FormRow {
	/// Term expression.
	pub expression: String,

	/// Kana reading for this term.
	pub reading: String,

	/// Kana reading as romaji.
	pub romaji: String,
}

/// Linked resource in the form.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkRow {
	pub uri:   String,
	pub title: String,
}

/// Index for a [SourceRow] in a [Root].
#[derive(Clone, Copy, Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct SourceId(usize);

impl SourceId {
	fn as_index(&self) -> usize {
		let SourceId(index) = self;
		*index
	}
}

/// Available sources for [KanjiRow] and [TermRow].
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SourceRow {
	/// Source name.
	pub name: String,

	/// Source revision version.
	pub revision: String,
}

/// Frequency metadata for kanji or terms.
#[derive(Serialize, Deserialize, Debug)]
pub struct MetaRow {
	/// Term or kanji.
	pub expression: String,

	/// Frequency value.
	pub value: u64,
}

/// Index for a [TagRow] in a [Root].
#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct TagId(usize);

impl TagId {
	fn as_index(&self) -> usize {
		let TagId(index) = self;
		*index
	}
}

/// Tag for an [KanjiRow] or [TermRow].
///
/// For kanji, this is also used to describe the `stats` keys.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TagRow {
	/// Short display name for this tag.
	pub name: String,

	/// Category name for this tag.
	pub category: String,

	/// Longer display description for this tag.
	pub description: String,

	/// Order value for this tag (lesser values sorted first).
	pub order: i32,

	/// Tag source dictionary.
	pub source: SourceId,
}

//
// Display implementation
//

impl DbDisplay for TermRow {
	fn fmt<W: std::io::Write>(&self, root: &Root, f: &mut W) -> io::Result<()> {
		write!(f, "{}", self.expression)?;
		if self.reading.len() > 0 {
			write!(f, " 「{}」", self.reading)?;
		}

		write!(f, " -- score:{}", self.score)?;

		if let Some(frequency) = self.frequency {
			write!(f, ", freq:{}", frequency)?;
		}

		let origin = &root.sources[self.source.as_index()];
		if origin.name.len() > 0 {
			write!(f, ", from:{}", origin.name)?;
		}

		write_tags(&self.tags, "\n   ", root, f)?;

		for (i, it) in self.definition.iter().enumerate() {
			write!(f, "\n\n   {}. ", i + 1)?;
			it.fmt(root, f)?;
		}

		if self.forms.len() > 0 {
			write!(f, "\n\n   ## Other forms ##\n")?;
			for it in &self.forms {
				write!(f, "\n   - {}", it.expression)?;
				if it.reading.len() > 0 {
					write!(f, " 「{}」", it.reading)?;
				}
			}
		}

		Ok(())
	}
}

impl DbDisplay for DefinitionRow {
	fn fmt<W: std::io::Write>(&self, root: &Root, f: &mut W) -> io::Result<()> {
		write!(f, "{}", self.text.join(", "))?;
		if self.info.len() > 0 {
			write!(f, " -- {}", self.info.join(", "))?;
		}

		write_tags(&self.tags, "\n   ", root, f)?;

		for it in &self.link {
			write!(f, "\n   - {}", it.uri)?;
			if it.title.len() > 0 {
				write!(f, " ({})", it.title)?;
			}
		}

		Ok(())
	}
}

impl DbDisplay for KanjiRow {
	fn fmt<W: std::io::Write>(&self, root: &Root, f: &mut W) -> io::Result<()> {
		write!(f, "{}", self.character)?;
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

		if let Some(frequency) = self.frequency {
			write!(f, " -- freq:{}", frequency)?;
		}

		write_tags(&self.tags, "\n   ", root, f)?;

		write!(f, "\n\n   {}", self.meanings.join("; "))?;

		if self.stats.len() > 0 {
			let pairs = self
				.stats
				.iter()
				.map(|(tag_id, value)| (&root.tags[tag_id.as_index()].name, value))
				.sorted();
			write!(f, "\n")?;
			for (i, (key, val)) in pairs.enumerate() {
				if i % 4 == 0 {
					write!(f, "\n   ")?;
				} else {
					write!(f, "      ")?;
				}
				write!(f, "|| {:16} | {:^10} ||", key, val)?;
			}
		}
		Ok(())
	}
}

impl fmt::Display for TagRow {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let SourceId(source) = self.source;
		write!(
			f,
			"{name:16} | {desc:75} | {cat:14} | {order:3} | {src}",
			cat = self.category,
			name = self.name,
			desc = if self.description.len() > 0 {
				self.description.as_str()
			} else {
				"--"
			},
			order = self.order,
			src = source + 1
		)
	}
}

impl fmt::Display for MetaRow {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{} = {}", self.expression, self.value)
	}
}

impl fmt::Display for SourceRow {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let revision = if self.revision.len() > 0 {
			format!(" ({})", self.revision)
		} else {
			String::new()
		};
		write!(f, "{}{}", self.name, revision)
	}
}

fn write_tags<L, T, W>(tags: L, prefix: &str, root: &Root, f: &mut W) -> io::Result<()>
where
	L: IntoIterator<Item = T>,
	T: std::borrow::Borrow<TagId>,
	W: std::io::Write,
{
	let tags = tags
		.into_iter()
		.map(|t| &root.tags[t.borrow().as_index()])
		.collect::<Vec<_>>();
	if tags.len() > 0 {
		write!(f, "{}[{}]", prefix, tags.iter().map(|it| &it.name).join(", "))?;
	}
	Ok(())
}

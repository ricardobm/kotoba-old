use std::borrow::Cow;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::Write;
use std::iter::IntoIterator;
use std::path::Path;

use itertools::Itertools;
use rand::prelude::SliceRandom;

use super::strings::{StringIndex, StringTable};

/// Dictionary of Japanese entries.
pub struct Dict {
	strings: StringTable,
	entries: Vec<EntryData>,
	tags:    Vec<TagData>,
}

const TAGS_FILE: &'static str = "tags.dat";
const STRINGS_FILE: &'static str = "strings.dat";
const ENTRIES_FILE: &'static str = "entries.dat";

impl Dict {
	/// All dictionary entries as a vector.
	pub fn entries<'a>(&'a self) -> Vec<Entry<'a>> {
		self.entries.iter().map(|it| Entry::from(self, it)).collect()
	}

	/// Number of entries in the dictionary.
	pub fn count(&self) -> usize {
		self.entries.len()
	}

	/// Shuffle all entries randomly.
	pub fn shuffle(&mut self, rng: &mut rand::prelude::ThreadRng) {
		self.entries.as_mut_slice().shuffle(rng);
	}

	/// Helper internal method to get a string from the internal [StringTable].
	pub(self) fn string<'a>(&'a self, index: StringIndex) -> &'a str {
		self.strings.get(index)
	}

	/// Helper internal method to get a [Tag] from a [TagId].
	pub(self) fn tag<'a>(&'a self, tag_id: TagId) -> Tag<'a> {
		let TagId(index) = tag_id;
		Tag::from(self, &self.tags[index])
	}

	/// Save the dictionary data to the given path.
	pub fn save(&self, base_path: &Path) -> io::Result<()> {
		let mut strings_file = base_path.to_path_buf();
		strings_file.push(STRINGS_FILE);
		let mut entries_file = base_path.to_path_buf();
		entries_file.push(ENTRIES_FILE);
		let mut tags_file = base_path.to_path_buf();
		tags_file.push(TAGS_FILE);

		let fs = File::create(&entries_file)?;
		let fs = io::BufWriter::new(fs);
		if let Err(err) = bincode::serialize_into(fs, &self.entries) {
			return io::Result::Err(io::Error::new(io::ErrorKind::Other, err));
		}

		let fs = File::create(&tags_file)?;
		let fs = io::BufWriter::new(fs);
		if let Err(err) = bincode::serialize_into(fs, &self.tags) {
			return io::Result::Err(io::Error::new(io::ErrorKind::Other, err));
		}

		let fs = File::create(&strings_file)?;
		let mut fs = io::BufWriter::new(fs);
		let mut first = true;
		for it in self.strings.entries() {
			if !first {
				fs.write("\n".as_bytes())?;
			}
			fs.write(it.as_bytes())?;
			first = false;
		}

		Ok(())
	}

	/// Load the dictionary data from the given path.
	pub fn load(base_path: &Path) -> io::Result<Dict> {
		let mut strings_file = base_path.to_path_buf();
		strings_file.push(STRINGS_FILE);
		let mut entries_file = base_path.to_path_buf();
		entries_file.push(ENTRIES_FILE);
		let mut tags_file = base_path.to_path_buf();
		tags_file.push(TAGS_FILE);

		let entries_file = File::open(entries_file)?;
		let entries_file = io::BufReader::new(entries_file);
		let tags_file = File::open(tags_file)?;
		let tags_file = io::BufReader::new(tags_file);

		let mut out = Dict {
			strings: StringTable::new(),

			entries: match bincode::deserialize_from(entries_file) {
				Ok(val) => val,
				Err(err) => {
					return io::Result::Err(io::Error::new(io::ErrorKind::Other, err));
				}
			},

			tags: match bincode::deserialize_from(tags_file) {
				Ok(val) => val,
				Err(err) => {
					return io::Result::Err(io::Error::new(io::ErrorKind::Other, err));
				}
			},
		};

		let data = std::fs::read_to_string(strings_file)?;
		for line in data.lines() {
			out.strings.load(line);
		}

		Ok(out)
	}
}

/// Dictionary entry.
#[derive(Copy, Clone)]
pub struct Entry<'a> {
	dict: &'a Dict,
	data: &'a EntryData,
}

impl<'a> Entry<'a> {
	pub(self) fn from(dict: &'a Dict, data: &'a EntryData) -> Entry<'a> {
		Entry { dict, data }
	}

	/// Additional origin information for this entry (human readable). The exact
	/// format and information depend on the source.
	pub fn origin(&'a self) -> &str {
		self.dict.string(self.data.origin)
	}

	/// Japanese expressions for this entry. The first entry is the main form.
	pub fn expressions(&'a self) -> Vec<&'a str> {
		self.data.expressions.iter().map(|&x| self.dict.string(x)).collect()
	}

	/// Respective kana readings for the `expressions`. An entry may be the
	/// empty string if the expression itself is already kana, or if a reading
	/// is not applicable.
	pub fn readings(&'a self) -> Vec<&'a str> {
		self.data.readings.iter().map(|&x| self.dict.string(x)).collect()
	}

	/// English definitions for this entry.
	pub fn definition(&'a self) -> Vec<Definition<'a>> {
		self.data
			.definitions
			.iter()
			.map(|it| Definition::from(self.dict, it))
			.collect()
	}

	/// Tags that apply to the entry itself. Possible examples are JLPT
	/// level, if the term is common, frequency information, etc.
	pub fn tags(&'a self) -> Vec<Tag<'a>> {
		self.data.tags.iter().map(|&it| self.dict.tag(it)).collect()
	}

	/// Numeric score of this entry (in case of multiple possibilities).
	///
	/// Higher values appear first. This does not affect entry with different
	/// origins.
	pub fn score(&'a self) -> i32 {
		self.data.score
	}
}

impl<'a> fmt::Display for Entry<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let expressions = self.expressions();
		let readings = self.readings();
		write!(f, "==> {}", expressions[0])?;
		if readings[0].len() > 0 {
			write!(f, " 「{}」", readings[0])?;
		}

		write!(f, " -- score:{}", self.score())?;
		let origin = self.origin();
		if origin.len() > 0 {
			write!(f, ", from:{}", origin)?;
		}

		let tags = self.tags();
		if tags.len() > 0 {
			write!(f, "\n[{}]", tags.iter().map(|it| it.name()).join(", "))?;
		}

		for (i, it) in self.definition().into_iter().enumerate() {
			write!(f, "\n\n{}. {}", i + 1, it)?;
		}

		if expressions.len() > 1 {
			write!(f, "\n\n## Other forms ##\n")?;
			for (i, it) in expressions.into_iter().enumerate().skip(1) {
				let reading = readings[i];
				write!(f, "\n- {}", it)?;
				if reading.len() > 0 {
					write!(f, " 「{}」", reading)?;
				}
			}
		}

		Ok(())
	}
}

/// English definition for an Entry.
#[derive(Copy, Clone)]
pub struct Definition<'a> {
	dict: &'a Dict,
	data: &'a DefinitionData,
}

impl<'a> Definition<'a> {
	pub(self) fn from(dict: &'a Dict, data: &'a DefinitionData) -> Definition<'a> {
		Definition { dict, data }
	}

	/// List of glossary terms for the meaning.
	pub fn glossary(&'a self) -> Vec<&'a str> {
		self.data.glossary.iter().map(|&it| self.dict.string(it)).collect()
	}

	/// Tags that apply to this meaning. Examples are: parts of speech, names,
	/// usage, area of knowledge, etc.
	pub fn tags(&'a self) -> Vec<Tag<'a>> {
		self.data.tags.iter().map(|&it| self.dict.tag(it)).collect()
	}

	/// Additional information to append to the entry definition.
	pub fn info(&'a self) -> Vec<&'a str> {
		self.data.info.iter().map(|&it| self.dict.string(it)).collect()
	}

	/// Related links. Those can be web URLs or other related words.
	pub fn links(&'a self) -> Vec<EntryLink<'a>> {
		self.data
			.links
			.iter()
			.map(|it| EntryLink::from(self.dict, it))
			.collect()
	}
}

impl<'a> fmt::Display for Definition<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.glossary().join(", "))?;
		let info = self.info();
		if info.len() > 0 {
			write!(f, " -- {}", info.join(", "))?;
		}

		let tags = self.tags();
		if tags.len() > 0 {
			write!(f, "\n[{}]", tags.iter().map(|it| it.name()).join(", "))?;
		}

		for it in self.links() {
			write!(f, "\n- {}", it)?;
		}

		Ok(())
	}
}

/// Link to related resources or entries.
#[derive(Copy, Clone)]
pub struct EntryLink<'a> {
	dict: &'a Dict,
	data: &'a LinkData,
}

impl<'a> EntryLink<'a> {
	pub(self) fn from(dict: &'a Dict, data: &'a LinkData) -> EntryLink<'a> {
		EntryLink { dict, data }
	}

	/// URI for the linked resource.
	pub fn uri(&'a self) -> &'a str {
		self.dict.string(self.data.uri)
	}

	/// Text for this link.
	pub fn text(&'a self) -> &'a str {
		self.dict.string(self.data.text)
	}
}

impl<'a> fmt::Display for EntryLink<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}: {}", self.uri(), self.text())
	}
}

/// Tag descriptor for a term or kanji in the dictionary.
#[derive(Copy, Clone)]
pub struct Tag<'a> {
	dict: &'a Dict,
	data: &'a TagData,
}

#[allow(dead_code)]
impl<'a> Tag<'a> {
	pub(self) fn from(dict: &'a Dict, data: &'a TagData) -> Tag<'a> {
		Tag { dict, data }
	}

	/// Short key for this tag that is used in the terms.
	pub fn name(&'a self) -> &'a str {
		self.dict.string(self.data.name)
	}

	/// Category name for this tag. Can be used to group tags by usage.
	pub fn category(&'a self) -> &'a str {
		self.dict.string(self.data.category)
	}

	/// Human readable description for the tag.
	pub fn description(&'a self) -> &'a str {
		self.dict.string(self.data.description)
	}

	/// Sorting value for the tag. Lower values mean higher precedence.
	pub fn order(&'a self) -> i32 {
		self.data.order
	}
}

/// Handle to reference a tag both internally and with builders.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct TagId(usize);

/// Builder used to construct the entries for a [Dict].
pub struct DictBuilder {
	strings: StringTable,
	entries: Vec<EntryData>,
	tags:    Vec<TagData>,
}

#[allow(dead_code)]
impl DictBuilder {
	pub fn new() -> DictBuilder {
		DictBuilder {
			strings: StringTable::new(),
			entries: Vec::new(),
			tags:    Vec::new(),
		}
	}

	/// Generate a new Dict with the entries in the builder.
	pub fn build(self) -> Dict {
		Dict {
			strings: self.strings,
			entries: self.entries,
			tags:    self.tags,
		}
	}

	/// Register a new tag and returns its [TagId] handle.
	pub fn add_tag<'a, F, S>(&mut self, tag_name: S, config: F) -> TagId
	where
		F: FnOnce(&mut DictBuilder, &mut TagBuilder<'a>),
		S: Into<Cow<'a, str>>,
	{
		let mut tag = TagBuilder {
			name:        tag_name.into(),
			category:    Cow::default(),
			description: Cow::default(),
			order:       0,
		};
		config(self, &mut tag);
		self.do_add_tag(tag)
	}

	/// Add a new entry to the builder.
	pub fn add_entry<'a, F, S>(&mut self, origin: S, config: F)
	where
		F: FnOnce(&mut DictBuilder, &mut EntryBuilder<'a>),
		S: Into<Cow<'a, str>>,
	{
		let mut entry = EntryBuilder::new(origin);
		config(self, &mut entry);
		self.do_add_entry(entry);
	}

	fn do_add_entry<'a>(&mut self, entry: EntryBuilder<'a>) {
		self.entries.push(EntryData::from_builder(&mut self.strings, entry));
	}

	fn do_add_tag<'a>(&mut self, tag: TagBuilder<'a>) -> TagId {
		self.tags.push(TagData::from_builder(&mut self.strings, tag));
		TagId(self.tags.len() - 1)
	}
}

/// Trait implemented by the builder that allow tags.
pub trait WithTags {
	/// Add a registered tag to the current entry.
	fn with_tag(&mut self, tag: TagId) -> &mut Self;

	/// Add a list of registered tags to the current entry.
	fn with_tags<L>(&mut self, tags: L) -> &mut Self
	where
		L: IntoIterator<Item = TagId>;
}

/// Builder to configure a dictionary entry.
#[allow(dead_code)]
pub struct EntryBuilder<'a> {
	pub(self) origin:      Cow<'a, str>,
	pub(self) expressions: Vec<Cow<'a, str>>,
	pub(self) readings:    Vec<Cow<'a, str>>,
	pub(self) definitions: Vec<DefinitionBuilder<'a>>,
	pub(self) tags:        Vec<TagId>,
	pub(self) score:       i32,
}

#[allow(dead_code)]
impl<'a> EntryBuilder<'a> {
	pub(self) fn new<S: Into<Cow<'a, str>>>(origin: S) -> EntryBuilder<'a> {
		EntryBuilder {
			origin:      origin.into(),
			expressions: Vec::new(),
			readings:    Vec::new(),
			definitions: Vec::new(),
			tags:        Vec::new(),
			score:       0,
		}
	}

	/// Add an expression to the entry. See [Entry::expression].
	pub fn add_expression<S: Into<Cow<'a, str>>>(&mut self, expr: S) -> &mut Self {
		self.expressions.push(expr.into());
		self
	}

	/// Add a list of expressions to the entry. See [Entry::expression].
	pub fn append_expressions<L, S>(&mut self, expr: L) -> &mut Self
	where
		L: IntoIterator<Item = S>,
		S: Into<Cow<'a, str>>,
	{
		for it in expr.into_iter() {
			self.expressions.push(it.into());
		}
		self
	}

	/// Add a reading to the entry. See [Entry::reading].
	pub fn add_reading<S: Into<Cow<'a, str>>>(&mut self, expr: S) -> &mut Self {
		self.readings.push(expr.into());
		self
	}

	/// Add a list of readings to the entry. See [Entry::reading].
	pub fn append_readings<L, S>(&mut self, expr: L) -> &mut Self
	where
		L: IntoIterator<Item = S>,
		S: Into<Cow<'a, str>>,
	{
		for it in expr.into_iter() {
			self.readings.push(it.into());
		}
		self
	}

	/// Set the entry score. See [Entry::score].
	pub fn with_score(&mut self, score: i32) -> &mut Self {
		self.score = score;
		self
	}

	/// Add a definition to the entry. See [Definition].
	pub fn add_definition<F, S>(&mut self, dict: &mut DictBuilder, glossary: S, config: F) -> &mut Self
	where
		F: FnOnce(&mut DictBuilder, &mut DefinitionBuilder<'a>),
		S: Into<Cow<'a, str>>,
	{
		let mut item = DefinitionBuilder {
			glossary: vec![glossary.into()],
			tags:     Vec::new(),
			info:     Vec::new(),
			links:    Vec::new(),
		};
		config(dict, &mut item);
		self.definitions.push(item);
		self
	}
}

impl<'a> WithTags for EntryBuilder<'a> {
	fn with_tag(&mut self, tag: TagId) -> &mut Self {
		self.tags.push(tag);
		self
	}

	fn with_tags<L>(&mut self, tags: L) -> &mut Self
	where
		L: IntoIterator<Item = TagId>,
	{
		for tag in tags.into_iter() {
			self.tags.push(tag);
		}
		self
	}
}

/// Builder to configure a [Definition] for an [Entry].
#[allow(dead_code)]
pub struct DefinitionBuilder<'a> {
	pub(self) glossary: Vec<Cow<'a, str>>,
	pub(self) tags:     Vec<TagId>,
	pub(self) info:     Vec<Cow<'a, str>>,
	pub(self) links:    Vec<EntryLinkBuilder<'a>>,
}

#[allow(dead_code)]
impl<'a> DefinitionBuilder<'a> {
	/// Add a glossary entry for the definition. See [Definition::glossary].
	pub fn add_glossary<S: Into<Cow<'a, str>>>(&mut self, expr: S) -> &mut Self {
		self.glossary.push(expr.into());
		self
	}

	/// Add a list of glossary entries for the definition. See [Definition::glossary].
	pub fn append_glossary<L, S>(&mut self, expr: L) -> &mut Self
	where
		L: IntoIterator<Item = S>,
		S: Into<Cow<'a, str>>,
	{
		for it in expr.into_iter() {
			self.glossary.push(it.into());
		}
		self
	}

	/// Add an information entry for the definition. See [Definition::info].
	pub fn add_info<S: Into<Cow<'a, str>>>(&mut self, expr: S) -> &mut Self {
		self.info.push(expr.into());
		self
	}

	/// Add a list of information entries for the definition. See [Definition::info].
	pub fn append_info<L, S>(&mut self, expr: L) -> &mut Self
	where
		L: IntoIterator<Item = S>,
		S: Into<Cow<'a, str>>,
	{
		for it in expr.into_iter() {
			self.info.push(it.into());
		}
		self
	}

	/// Add a link to this definition. See [Definition::link].
	pub fn add_link<S: Into<Cow<'a, str>>>(&mut self, uri: S, text: S) -> &mut Self {
		self.links.push(EntryLinkBuilder {
			uri:  uri.into(),
			text: text.into(),
		});
		self
	}
}

impl<'a> WithTags for DefinitionBuilder<'a> {
	fn with_tag(&mut self, tag: TagId) -> &mut Self {
		self.tags.push(tag);
		self
	}

	fn with_tags<L>(&mut self, tags: L) -> &mut Self
	where
		L: IntoIterator<Item = TagId>,
	{
		for tag in tags.into_iter() {
			self.tags.push(tag);
		}
		self
	}
}

/// Used internally to keep link data during building.
struct EntryLinkBuilder<'a> {
	pub uri:  Cow<'a, str>,
	pub text: Cow<'a, str>,
}

/// Builder to configure a [Tag].
pub struct TagBuilder<'a> {
	pub(self) name:        Cow<'a, str>,
	pub(self) category:    Cow<'a, str>,
	pub(self) description: Cow<'a, str>,
	pub(self) order:       i32,
}

#[allow(dead_code)]
impl<'a> TagBuilder<'a> {
	/// Set the tag's category. See [Tag::category].
	pub fn with_category<S: Into<Cow<'a, str>>>(&mut self, category: S) -> &mut Self {
		self.category = category.into();
		self
	}

	/// Set the tag's description. See [Tag::description].
	pub fn with_description<S: Into<Cow<'a, str>>>(&mut self, description: S) -> &mut Self {
		self.description = description.into();
		self
	}

	/// Set the tag's order. See [Tag::order].
	pub fn with_order(&mut self, order: i32) -> &mut Self {
		self.order = order;
		self
	}
}

//
// INTERNAL DATA
// =============
//
// Structures used to keep the dictionary data internally. All strings are
// stored with their [Dict::strings] ids and tags are referenced by [TagId].
//

#[derive(Serialize, Deserialize)]
struct EntryData {
	origin:      StringIndex,
	expressions: Vec<StringIndex>,
	readings:    Vec<StringIndex>,
	definitions: Vec<DefinitionData>,
	tags:        Vec<TagId>,
	score:       i32,
}

impl<'a> EntryData {
	pub fn from_builder(strings: &mut StringTable, builder: EntryBuilder<'a>) -> EntryData {
		EntryData {
			origin:      strings.intern(builder.origin),
			expressions: builder.expressions.into_iter().map(|it| strings.intern(it)).collect(),
			readings:    builder.readings.into_iter().map(|it| strings.intern(it)).collect(),
			definitions: builder
				.definitions
				.into_iter()
				.map(|it| DefinitionData::from_builder(strings, it))
				.collect(),
			tags:        builder.tags,
			score:       builder.score,
		}
	}
}

#[derive(Serialize, Deserialize)]
struct DefinitionData {
	glossary: Vec<StringIndex>,
	tags:     Vec<TagId>,
	info:     Vec<StringIndex>,
	links:    Vec<LinkData>,
}

impl<'a> DefinitionData {
	pub fn from_builder(strings: &mut StringTable, builder: DefinitionBuilder<'a>) -> DefinitionData {
		DefinitionData {
			glossary: builder.glossary.into_iter().map(|it| strings.intern(it)).collect(),
			tags:     builder.tags,
			info:     builder.info.into_iter().map(|it| strings.intern(it)).collect(),
			links:    builder
				.links
				.into_iter()
				.map(|it| LinkData {
					uri:  strings.intern(it.uri),
					text: strings.intern(it.text),
				})
				.collect(),
		}
	}
}

#[derive(Serialize, Deserialize)]
struct LinkData {
	uri:  StringIndex,
	text: StringIndex,
}

#[derive(Serialize, Deserialize)]
struct TagData {
	name:        StringIndex,
	category:    StringIndex,
	description: StringIndex,
	order:       i32,
}

impl<'a> TagData {
	pub fn from_builder(strings: &mut StringTable, builder: TagBuilder<'a>) -> TagData {
		TagData {
			name:        strings.intern(builder.name),
			category:    strings.intern(builder.category),
			description: strings.intern(builder.description),
			order:       builder.order,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_dict_builder() {
		let mut builder = DictBuilder::new();

		let tag_a1 = builder.add_tag("tag A1", |_, tag: &mut TagBuilder| {
			tag.with_category("cat A1").with_description("desc A1").with_order(101);
		});
		let tag_a2 = builder.add_tag("tag A2", |_, tag: &mut TagBuilder| {
			tag.with_category("cat A2").with_description("desc A2").with_order(101);
		});
		let tag_a3 = builder.add_tag("tag A3", |_, tag: &mut TagBuilder| {
			tag.with_category("cat A3").with_description("desc A3").with_order(101);
		});

		let tag_b1 = builder.add_tag("tag B1", |_, tag: &mut TagBuilder| {
			tag.with_category("cat B1").with_description("desc B1").with_order(101);
		});
		let tag_b2 = builder.add_tag("tag B2", |_, tag: &mut TagBuilder| {
			tag.with_category("cat B2").with_description("desc B2").with_order(101);
		});
		let tag_b3 = builder.add_tag("tag B3", |_, tag: &mut TagBuilder| {
			tag.with_category("cat B3").with_description("desc B3").with_order(101);
		});

		builder.add_entry("some origin", |builder, entry: &mut EntryBuilder| {
			entry
				.add_expression("expr 1")
				.append_expressions(vec!["expr 2", "expr 3"])
				.add_expression("expr 4");
			entry
				.add_reading("read 1")
				.append_readings(vec!["read 2", "read 3"])
				.add_reading("read 4");
			entry.with_score(123);
			entry.add_definition(builder, "word 1", |_, def: &mut DefinitionBuilder| {
				def.add_glossary("word 2")
					.append_glossary(vec!["word 3", "word 4"])
					.add_glossary("word 5");
				def.add_info("info 1")
					.append_info(vec!["info 2", "info 3"])
					.add_info("info 4");
				def.add_link("uri 1", "text 1")
					.add_link("uri 2", "text 2")
					.add_link("uri 3", "text 3");
				def.with_tag(tag_b1).with_tags(vec![tag_b2, tag_b3]);
			});
			entry.with_tag(tag_a1).with_tags(vec![tag_a2, tag_a3]);
		})
	}
}

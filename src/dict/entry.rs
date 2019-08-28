use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::iter::IntoIterator;

use itertools::Itertools;
use rand::prelude::SliceRandom;

struct InternalData {
	str_table: Vec<String>,
	str_index: HashMap<InternalString, usize>,
}

type StringIndex = usize;

impl<'a> InternalData {
	pub fn new() -> InternalData {
		let mut result = InternalData {
			str_table: Vec::new(),
			str_index: HashMap::new(),
		};
		result.intern(String::new());
		result
	}

	pub fn intern<S: Into<Cow<'a, str>>>(&mut self, value: S) -> StringIndex {
		let cow = value.into();
		let key = InternalString::from(&cow);
		if let Some(&index) = self.str_index.get(&key) {
			index
		} else {
			// Push a new entry into the table.
			self.str_table.push(cow.into());

			// Generate a new key pointing to the entry in the table.
			let key = InternalString::from(self.str_table.last().unwrap().as_str());
			self.str_index.insert(key, self.str_table.len() - 1);
			self.str_table.len() - 1
		}
	}
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
struct InternalString {
	ptr: *const str,
}

#[allow(dead_code)]
impl InternalString {
	fn from<S>(value: S) -> InternalString
	where
		S: AsRef<str>,
	{
		let ptr = value.as_ref() as *const str;
		InternalString { ptr }
	}
}

impl std::cmp::PartialEq for InternalString {
	fn eq(&self, other: &Self) -> bool {
		if self.ptr == other.ptr {
			true
		} else {
			unsafe { (*self.ptr) == (*other.ptr) }
		}
	}
}

impl std::cmp::Eq for InternalString {}

impl std::hash::Hash for InternalString {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		unsafe { (*self.ptr).hash(state) }
	}
}

pub struct DictBuilder {
	internal: InternalData,
	entries:  Vec<EntryData>,
	tags:     Vec<TagData>,
}

pub type TagId = usize;

#[allow(dead_code)]
impl DictBuilder {
	pub fn new() -> DictBuilder {
		DictBuilder {
			internal: InternalData::new(),
			entries:  Vec::new(),
			tags:     Vec::new(),
		}
	}

	pub fn build(self) -> Dict {
		Dict {
			internal: self.internal,
			entries:  self.entries,
			tags:     self.tags,
		}
	}

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

	pub fn add_entry<'a, F, S>(&mut self, source: EntrySource, origin: S, config: F)
	where
		F: FnOnce(&mut DictBuilder, &mut EntryBuilder<'a>),
		S: Into<Cow<'a, str>>,
	{
		let mut entry = EntryBuilder::new(source, origin);
		config(self, &mut entry);
		self.do_add_entry(entry);
	}

	fn do_add_entry<'a>(&mut self, entry: EntryBuilder<'a>) {
		self.entries.push(EntryData::from_builder(&mut self.internal, entry));
	}

	fn do_add_tag<'a>(&mut self, tag: TagBuilder<'a>) -> TagId {
		self.tags.push(TagData::from_builder(&mut self.internal, tag));
		self.tags.len() - 1
	}
}

pub trait WithTags {
	fn with_tag(&mut self, tag: TagId) -> &mut Self;

	fn with_tags<L>(&mut self, tags: L) -> &mut Self
	where
		L: IntoIterator<Item = TagId>;
}

/// Dictionary entry.
#[allow(dead_code)]
pub struct EntryBuilder<'a> {
	/// Source for this entry.
	pub(self) source: EntrySource,

	/// Additional origin information for this entry (human readable). The exact
	/// format and information depend on the source.
	pub(self) origin: Cow<'a, str>,

	/// Japanese expressions for this entry. The first entry is the main form.
	pub(self) expressions: Vec<Cow<'a, str>>,

	/// Respective kana readings for the `expressions`. An entry may be the
	/// empty string if the expression itself is already kana, or if a reading
	/// is not applicable.
	pub(self) readings: Vec<Cow<'a, str>>,

	/// English definitions for this entry.
	pub(self) definitions: Vec<DefinitionBuilder<'a>>,

	/// Tags that apply to the entry itself. Possible examples are JLPT
	/// level, if the term is common, frequency information, etc.
	pub(self) tags: Vec<TagId>,

	/// Numeric score of this entry (in case of multiple possibilities).
	///
	/// Higher values appear first. This does not affect entry with different
	/// origins.
	pub(self) score: i32,
}

struct EntryData {
	source:      EntrySource,
	origin:      StringIndex,
	expressions: Vec<StringIndex>,
	readings:    Vec<StringIndex>,
	definitions: Vec<DefinitionData>,
	tags:        Vec<TagId>,
	score:       i32,
}

impl<'a> EntryData {
	pub fn from_builder(internal: &mut InternalData, builder: EntryBuilder<'a>) -> EntryData {
		EntryData {
			source:      builder.source,
			origin:      internal.intern(builder.origin),
			expressions: builder.expressions.into_iter().map(|it| internal.intern(it)).collect(),
			readings:    builder.readings.into_iter().map(|it| internal.intern(it)).collect(),
			definitions: builder
				.definitions
				.into_iter()
				.map(|it| DefinitionData::from_builder(internal, it))
				.collect(),
			tags:        builder.tags,
			score:       builder.score,
		}
	}
}

#[allow(dead_code)]
impl<'a> EntryBuilder<'a> {
	pub(self) fn new<S: Into<Cow<'a, str>>>(source: EntrySource, origin: S) -> EntryBuilder<'a> {
		EntryBuilder {
			source:      source,
			origin:      origin.into(),
			expressions: Vec::new(),
			readings:    Vec::new(),
			definitions: Vec::new(),
			tags:        Vec::new(),
			score:       0,
		}
	}

	pub fn add_expression<S: Into<Cow<'a, str>>>(&mut self, expr: S) -> &mut Self {
		self.expressions.push(expr.into());
		self
	}

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

	pub fn add_reading<S: Into<Cow<'a, str>>>(&mut self, expr: S) -> &mut Self {
		self.readings.push(expr.into());
		self
	}

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

	pub fn with_score(&mut self, score: i32) -> &mut Self {
		self.score = score;
		self
	}

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

#[allow(dead_code)]
pub struct DefinitionBuilder<'a> {
	/// List of glossary terms for the meaning.
	pub glossary: Vec<Cow<'a, str>>,

	/// Tags that apply to this meaning. Examples are: parts of speech, names,
	/// usage, area of knowledge, etc.
	pub tags: Vec<TagId>,

	/// Additional information to append to the entry definition.
	pub info: Vec<Cow<'a, str>>,

	/// Related links. Those can be web URLs or other related words.
	pub links: Vec<EntryLinkInfo<'a>>,
}

struct DefinitionData {
	glossary: Vec<StringIndex>,
	tags:     Vec<TagId>,
	info:     Vec<StringIndex>,
	links:    Vec<LinkData>,
}

impl<'a> DefinitionData {
	pub fn from_builder(internal: &mut InternalData, builder: DefinitionBuilder<'a>) -> DefinitionData {
		DefinitionData {
			glossary: builder.glossary.into_iter().map(|it| internal.intern(it)).collect(),
			tags:     builder.tags,
			info:     builder.info.into_iter().map(|it| internal.intern(it)).collect(),
			links:    builder
				.links
				.into_iter()
				.map(|it| LinkData {
					uri:  internal.intern(it.uri),
					text: internal.intern(it.text),
				})
				.collect(),
		}
	}
}

struct LinkData {
	uri:  StringIndex,
	text: StringIndex,
}

#[allow(dead_code)]
impl<'a> DefinitionBuilder<'a> {
	pub fn add_glossary<S: Into<Cow<'a, str>>>(&mut self, expr: S) -> &mut Self {
		self.glossary.push(expr.into());
		self
	}

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

	pub fn add_info<S: Into<Cow<'a, str>>>(&mut self, expr: S) -> &mut Self {
		self.info.push(expr.into());
		self
	}

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

	pub fn add_link<S: Into<Cow<'a, str>>>(&mut self, uri: S, text: S) -> &mut Self {
		self.links.push(EntryLinkInfo {
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

pub struct EntryLinkInfo<'a> {
	/// URI for the linked resource.
	pub uri: Cow<'a, str>,

	/// Text for this link.
	pub text: Cow<'a, str>,
}

pub struct TagBuilder<'a> {
	/// Short key for this tag that is used in the terms.
	pub name: Cow<'a, str>,

	/// Category name for this tag. Can be used to group tags by usage.
	pub category: Cow<'a, str>,

	/// Human readable description for the tag.
	pub description: Cow<'a, str>,

	/// Sorting value for the tag. Lower values mean higher precedence.
	pub order: i32,
}

struct TagData {
	name:        StringIndex,
	category:    StringIndex,
	description: StringIndex,
	order:       i32,
}

impl<'a> TagData {
	pub fn from_builder(internal: &mut InternalData, builder: TagBuilder<'a>) -> TagData {
		TagData {
			name:        internal.intern(builder.name),
			category:    internal.intern(builder.category),
			description: internal.intern(builder.description),
			order:       builder.order,
		}
	}
}

#[allow(dead_code)]
impl<'a> TagBuilder<'a> {
	pub fn with_category<S: Into<Cow<'a, str>>>(&mut self, category: S) -> &mut Self {
		self.category = category.into();
		self
	}

	pub fn with_description<S: Into<Cow<'a, str>>>(&mut self, description: S) -> &mut Self {
		self.description = description.into();
		self
	}

	pub fn with_order(&mut self, order: i32) -> &mut Self {
		self.order = order;
		self
	}
}

pub struct Dict {
	internal: InternalData,
	entries:  Vec<EntryData>,
	tags:     Vec<TagData>,
}

impl Dict {
	pub fn entries<'a>(&'a self) -> Vec<Entry<'a>> {
		self.entries.iter().map(|it| Entry::from(self, it)).collect()
	}

	pub fn count(&self) -> usize {
		self.entries.len()
	}

	pub fn shuffle(&mut self, rng: &mut rand::prelude::ThreadRng) {
		self.entries.as_mut_slice().shuffle(rng);
	}

	pub(self) fn string<'a>(&'a self, index: StringIndex) -> &'a str {
		self.internal.str_table[index].as_str()
	}

	pub(self) fn tag<'a>(&'a self, tag_id: TagId) -> Tag<'a> {
		Tag::from(self, &self.tags[tag_id])
	}
}

/// Origin for a dictionary entry.
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum EntrySource {
	/// Entry was imported from a dictionary file.
	Import      = 0,
	/// Entry was imported from `jisho.org`.
	Jisho       = 1,
	/// Entry was imported from `japanesepod101.com`.
	JapanesePod = 2,
}

impl fmt::Display for EntrySource {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			EntrySource::Import => write!(f, "import"),
			EntrySource::Jisho => write!(f, "jisho"),
			EntrySource::JapanesePod => write!(f, "japanesepod101"),
		}
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

	/// Source for this entry.
	pub fn source(&'a self) -> EntrySource {
		self.data.source
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

		write!(f, " -- score:{}, source:{}", self.score(), self.source())?;
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

/// English meaning for an entry.
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

/// Link to related resources.
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

/// Tag for a term or kanji in the dictionary.
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

		builder.add_entry(
			EntrySource::Import,
			"some origin",
			|builder, entry: &mut EntryBuilder| {
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
			},
		)
	}
}

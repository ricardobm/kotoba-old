use std::fmt;

/// Origin for a dictionary entry.
pub enum EntrySource {
	/// Entry was imported from a dictionary file.
	Import,
	/// Entry was imported from `jisho.org`.
	Jisho,
	/// Entry was imported from `japanesepod101.com`.
	JapanesePod,
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
pub struct Entry {
	/// Source for this entry.
	pub source: EntrySource,

	/// Additional origin information for this entry (human readable). The exact
	/// format and information depend on the source.
	pub origin: String,

	/// Japanese expressions for this entry. The first entry is the main form.
	pub expressions: Vec<String>,

	/// Respective kana readings for the `expressions`. An entry may be the
	/// empty string if the expression itself is already kana, or if a reading
	/// is not applicable.
	pub readings: Vec<String>,

	/// English definitions for this entry.
	pub definition: Vec<EntryEnglish>,

	/// Tags that apply to the entry itself. Possible examples are JLPT
	/// level, if the term is common, frequency information, etc.
	pub tags: Vec<String>,

	/// Numeric score of this entry (in case of multiple possibilities).
	///
	/// Higher values appear first. This does not affect entry with different
	/// origins.
	pub score: i32,
}

impl fmt::Display for Entry {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "==> {}", self.expressions[0])?;
		if self.readings[0].len() > 0 {
			write!(f, " 「{}」", self.readings[1])?;
		}
		write!(f, " -- {}", self.score)?;

		if self.tags.len() > 0 {
			write!(f, "\n[{}]", self.tags.join(", "))?;
		}

		for (i, it) in self.definition.iter().enumerate() {
			write!(f, "\n\n{}. {}", i + 1, it)?;
		}

		if self.expressions.len() > 1 {
			write!(f, "\n\n## Other forms ##\n")?;
			for (i, it) in self.expressions.iter().enumerate().skip(1) {
				let reading = &self.readings[i];
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
pub struct EntryEnglish {
	/// List of glossary terms for the meaning.
	glossary: Vec<String>,

	/// Tags that apply to this meaning. Examples are: parts of speech, names,
	/// usage, area of knowledge, etc.
	tags: Vec<String>,

	/// Additional information to append to the entry definition.
	info: Vec<String>,

	/// Related links. Those can be web URLs or other related words.
	links: Vec<EntryLink>,
}

impl fmt::Display for EntryEnglish {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.glossary.join(", "))?;
		if self.info.len() > 0 {
			write!(f, " -- {}", self.info.join(", "))?;
		}
		if self.tags.len() > 0 {
			write!(f, "\n[{}]", self.tags.join(", "))?;
		}
		for it in &self.links {
			write!(f, "\n- {}", it)?;
		}
		Ok(())
	}
}

/// Link to related resources.
pub struct EntryLink {
	/// URI for the linked resource.
	uri: String,

	/// Text for this link.
	text: String,
}

impl fmt::Display for EntryLink {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}: {}", self.uri, self.text)
	}
}

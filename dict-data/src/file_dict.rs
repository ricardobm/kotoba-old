use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::files::Zip;
use crate::raw::RawUint32;

/// Implement loading for the `dict.zip` file which contains the main dictionary
/// entries.
///
/// The structure of this file consists of an `index` file and several numeric
/// files (`0000`, `0001`, `0002`...).
///
/// The `index` file contains a list of `EntryHeader` for each term in the
/// dictionary.
///
/// The contents for a given `EntryHeader` term can be retrieved from the numeric
/// file given by the row position in groups of 1000 (i.e. `index / 1000`).
pub struct Dict {
	rows: Vec<EntryHeader>,
	pages: Arc<Mutex<DictPages>>,
}

struct DictPages {
	source: Zip,
	cached: HashMap<usize, EntriesPage>,
}

impl Dict {
	pub fn new(mut data: Zip) -> std::io::Result<Dict> {
		let index: Vec<EntryHeader> = data.read_vec("index")?;
		Ok(Dict {
			rows: index,
			pages: Arc::new(Mutex::new(DictPages {
				source: data,
				cached: Default::default(),
			})),
		})
	}
}

impl Dict {
	pub fn count(&self) -> usize {
		self.rows.len()
	}

	pub fn get_entry(&self, index: usize) -> DictEntry {
		let page_number = index / 1000;
		let page_offset = index % 1000;

		let pages = Arc::clone(&self.pages);
		let mut pages = pages.lock().unwrap();
		if !pages.cached.contains_key(&page_number) {
			let page_file = format!("{:04}", page_number);
			let page_data = pages.source.read_vec(&page_file).unwrap();
			pages
				.cached
				.insert(page_number, EntriesPage { data: page_data });
		}
		let page = &pages.cached[&page_number];
		let data = page.get_entry(page_offset);

		let head = &self.rows[index];

		DictEntry {
			expression: head.expression.into(),
			reading: head.reading.into(),
			lookup: head.lookup.into(),
			frequency: head.frequency.into(),
			data: data,
		}
	}
}

pub struct DictEntry {
	pub expression: usize,
	pub reading: usize,
	pub lookup: usize,
	pub frequency: u32,
	data: EntryData,
}

/// A single entry from the `index` file.
#[repr(C, align(4))]
struct EntryHeader {
	/// Index of the main expression text for the entry. This is the main
	/// dictionary term.
	expression: RawUint32,

	/// Index of the reading text for the entry. The reading is a hiragana
	/// string for the expression.
	reading: RawUint32,

	/// Index of the lookup text for the entry. The lookup is an ASCII-only
	/// string generated from the romaji for the reading that helps with
	/// dictionary searching.
	lookup: RawUint32,

	/// Frequency number for the entry. The frequency is the number of
	/// occurrences of the main entry in a reference Japanese corpus text.
	///
	/// Rows in `index` are sorted by frequency.
	frequency: RawUint32,
}

/// EntriesPage represents the contents of a single numeric file from `Dict`.
///
/// A numeric file is composed entirely of 32-bit unsigned integers and its
/// purpose is to store the variable-length data for a term.
///
/// The format of a numeric file is:
///
/// ```
/// 	EntriesPage {
/// 		IndexLength: u32_le,
/// 		DataLength:  u32_le,
/// 		Index:       [u32_le; IndexLength],
/// 		Data:        [u32_le; DataLength],
/// 	}
/// ```
///
/// Each entry in `Index` corresponds to a term in the main `index` file, and
/// gives the offset for that entry's content in the `Data` array.
///
/// ```
/// 	EntryData {
/// 		DefinitionCount: u32_le,
/// 		Definitions:     [EntryDefinition; DefinitionCount],
/// 	}
///
/// 	EntryDefinition {
/// 		SourceIndex: u32_le,
/// 		Text:        EntryDefinitionList,
/// 		Rules:       EntryDefinitionList,
/// 		TagsForTerm: EntryDefinitionList,
/// 		TagsForText: EntryDefinitionList,
/// 	}
///
/// 	EntryDefinitionList {
/// 		Count: u32_le,
/// 		Items: [u32_le; Count],
/// 	}
/// ```
struct EntriesPage {
	// Entire data for the field.
	data: Vec<RawUint32>,
}

impl EntriesPage {
	/// Number of entries in this file.
	pub fn count(&self) -> usize {
		self.data[0].into()
	}

	/// Returns the data for an entry from the file.
	pub fn get_entry(&self, index: usize) -> EntryData {
		let count = self.count();
		let data_index = &self.data[2..2 + count];
		let data_block = &self.data[2 + count..];
		let data_offset: usize = data_index[index].into();

		let mut data = &data_block[data_offset..];

		let definition_count: usize = data[0].into();
		data = &data[1..];

		let mut entry = EntryData {
			definitions: Vec::with_capacity(definition_count),
		};

		for _ in 0..definition_count {
			let source = data[0];
			data = &data[1..];

			let text_length: usize = data[0].into();
			data = &data[1..];
			let text = &data[..text_length];
			data = &data[text_length..];

			let rules_length: usize = data[0].into();
			data = &data[1..];
			let rules = &data[..rules_length];
			data = &data[rules_length..];

			let tags_for_term_length: usize = data[0].into();
			data = &data[1..];
			let tags_for_term = &data[..tags_for_term_length];
			data = &data[tags_for_term_length..];

			let tags_for_text_length: usize = data[0].into();
			data = &data[1..];
			let tags_for_text = &data[..tags_for_text_length];
			data = &data[tags_for_text_length..];

			entry.definitions.push(EntryDefinition {
				source: source.into(),
				text: text.iter().map(|&x| x.into()).collect(),
				rules: rules.iter().map(|&x| x.into()).collect(),
				tags_for_term: tags_for_term.iter().map(|&x| x.into()).collect(),
				tags_for_text: tags_for_text.iter().map(|&x| x.into()).collect(),
			});
		}

		entry
	}
}

struct EntryData {
	definitions: Vec<EntryDefinition>,
}

struct EntryDefinition {
	source: usize,
	text: Vec<usize>,
	rules: Vec<usize>,
	tags_for_term: Vec<usize>,
	tags_for_text: Vec<usize>,
}

fn _assert_send_sync()
where
	Dict: Send + Sync,
{
}

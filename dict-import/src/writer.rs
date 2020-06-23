//! Output intermediate files with the compiled dictionary data.

use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::io::BufWriter;
use std::io::Result;
use std::io::Write;

use serde::Serialize;
use unicase::UniCase;
use unicode_segmentation::UnicodeSegmentation;

use crate::dict::{Dict, Kanji, Tag, Term};

#[derive(Default)]
pub struct Writer {
	/// Frequency map of terms to number of appearances.
	freq_terms: HashMap<String, u32>,

	/// Frequency map of kanji to number of appearances.
	freq_kanji: HashMap<String, u32>,

	/// List of terms from all dictionaries.
	terms: Vec<Term>,

	/// List of kanji from all dictionaries.
	kanji: Vec<Kanji>,

	/// Set of tags from all dictionaries by name.
	tag_map: HashMap<String, Tag>,
}

impl Writer {
	/// Append dictionary data into the dictionary.
	pub fn append_dict(&mut self, dict: Dict) {
		for it in dict.tags {
			self.append_tag(it);
		}

		for it in dict.meta_terms {
			self.freq_terms.insert(it.expression, it.data);
		}

		for it in dict.meta_kanji {
			self.freq_kanji.insert(it.expression, it.data);
		}

		for it in dict.terms {
			self.map_tags(it.term_tags.clone());
			self.map_tags(it.definition_tags.clone());
			self.map_tags(it.rules.clone());
			self.terms.push(it);
		}

		for it in dict.kanji {
			self.map_tags(it.tags.clone());
			self.map_tags(it.stats.keys().cloned().collect());
			self.kanji.push(it);
		}
	}

	/// Outputs all data to code files.
	pub fn output<P: AsRef<std::path::Path>>(self, output_directory: P) -> Result<()> {
		use regex::Regex;
		use std::time::Instant;

		lazy_static! {
			static ref RE_SPLIT_ENGLISH: Regex = Regex::new(r"(?i)[^\p{Alphabetic}]+").unwrap();
			static ref RE_ENGLISH_VALIDATE: Regex = Regex::new(r"^[a-z0-9]+$").unwrap();
		}

		//--------------------------------------------------------------------//
		// Data compilation
		//--------------------------------------------------------------------//

		println!("... compiling data");

		//
		// Tag handling:
		//

		let start = Instant::now();

		// Map of tag index to sorting key
		let mut tag_order = HashMap::new();

		// Map of tag name to index in tag list
		let mut tag_map = HashMap::new();

		// List of tags, sorted by name just for convenience
		let mut tags = Vec::new();

		let mut all_tags = self.tag_map.into_iter().collect::<Vec<_>>();
		all_tags.sort_by(|a, b| UniCase::new(&a.0).cmp(&UniCase::new(&b.0)));
		for (index, (key, tag)) in all_tags.into_iter().enumerate() {
			let tag = TagData {
				index: index,
				name: tag.name,
				category: tag.category,
				order: tag.order,
				notes: tag.notes,
			};
			tag_map.insert(key, index);
			tag_order.insert(index, (tag.order, UniCase::new(tag.name.clone())));
			tags.push(tag);
		}

		println!("... - compiled tags ({:?})", start.elapsed());

		// Helper to sort tag indexes.
		let sort_tag = |a: &usize, b: &usize| {
			let tag_a = &tag_order[a];
			let tag_b = &tag_order[b];
			tag_a.cmp(&tag_b)
		};

		// Helper to map tag name to index.
		let map_tags = |tags: &Vec<String>| {
			let mut out = tags
				.iter()
				.map(|x| tag_map.get(x).cloned().unwrap())
				.collect::<Vec<_>>();
			out.sort_by(sort_tag);
			out
		};

		//
		// Kanji:
		//

		let kanji_freq = self.freq_kanji;
		let mut kanji: Vec<_> = self
			.kanji
			.into_iter()
			.map(move |mut k| {
				k.frequency = kanji_freq
					.get(&k.character.to_string())
					.cloned()
					.unwrap_or_default();
				k
			})
			.collect();
		kanji.sort_by(|a, b| {
			if a.frequency != b.frequency {
				b.frequency.cmp(&a.frequency)
			} else {
				a.character.cmp(&b.character)
			}
		});

		//
		// String interning:
		//

		let start = Instant::now();

		let mut sources: HashSet<&str> = HashSet::new();
		let mut terms: HashSet<&str> = HashSet::new();
		let mut glossary: HashSet<&str> = HashSet::new();
		let mut search: HashSet<&str> = HashSet::new();

		let mut english: HashMap<String, HashSet<usize>> = HashMap::new();

		for term in self.terms.iter() {
			if term.source.len() > 0 {
				sources.insert(term.source.as_str());
			}
			if term.expression.len() > 0 {
				terms.insert(term.expression.as_str());
			}
			if term.reading.len() > 0 {
				terms.insert(term.reading.as_str());
			}
			if term.search_key.len() > 0 {
				search.insert(term.search_key.as_str());
			}
			for it in term.glossary.iter() {
				glossary.insert(it.as_str());
			}
		}

		// Generate the flat string list and a reverse lookup map of the string
		// to its index:

		fn string_hash_to_list<'a>(
			input: HashSet<&'a str>,
		) -> (Vec<&'a str>, HashMap<&'a str, usize>) {
			// Collect strings into a sorted list:
			let mut list: Vec<_> = input.into_iter().collect();
			list.sort();
			// Generate a reverse lookup map:
			let map: HashMap<_, _> = list
				.iter()
				.enumerate()
				.map(|(index, &s)| (s, index))
				.collect();
			(list, map)
		}

		let (sources, sources_map) = string_hash_to_list(sources);
		let (terms, terms_map) = string_hash_to_list(terms);
		let (glossary, glossary_map) = string_hash_to_list(glossary);
		let (search, search_map) = string_hash_to_list(search);

		// Validate that no term in the glossary has a line break, otherwise we
		// would break our output.
		for (index, it) in glossary.iter().enumerate() {
			if it.contains('\n') {
				panic!("Glossary term contain like break at {}: {}", index + 1, it);
			}
		}

		println!("... - interned strings ({:?})", start.elapsed());

		//
		// Term dictionary:
		//

		let start = Instant::now();

		let get_index =
			|m: &HashMap<&str, usize>, key: &str| if key.len() == 0 { 0 } else { m[key] + 1 };

		// First index dictionary terms by `(expression, reading)`. This already
		// interns the strings and maps tags:
		let freq_terms = self.freq_terms;
		let mut term_map: HashMap<(usize, usize), TermData> = HashMap::new();
		for term in self.terms.iter() {
			let expr = get_index(&terms_map, &term.expression);
			let read = get_index(&terms_map, &term.reading);
			let key = (expr, read);
			let data = term_map.entry(key).or_insert_with(|| TermData {
				expr: expr,
				read: read,
				look: get_index(&search_map, &term.search_key),
				freq: freq_terms
					.get(&term.expression)
					.cloned()
					.unwrap_or_default(),
				defs: Vec::new(),
			});
			data.defs.push(DefinitionData {
				tags_term: map_tags(&term.term_tags),
				tags_text: map_tags(&term.definition_tags),
				text: term
					.glossary
					.iter()
					.map(|x| get_index(&glossary_map, x))
					.collect(),
				rules: map_tags(&term.rules),
				source: get_index(&sources_map, &term.source),
				score: term.score,
			});
		}

		// Flatten the dictionary map into a vector sorted by the indexes:
		let mut dictionary = term_map.into_iter().collect::<Vec<_>>();
		dictionary.sort_by(|a, b| a.0.cmp(&b.0));

		let dictionary = dictionary.into_iter().map(|x| x.1).collect::<Vec<_>>();
		println!("... - compiled dictionary data ({:?})", start.elapsed());

		//
		// Terms and search index:
		//

		let start = Instant::now();

		// Maps Japanese characters to the respective entry indexes.
		let mut chars_index: HashMap<char, HashSet<usize>> = HashMap::new();

		// Append the given entry index and text to a `chars_index` map.
		let append_char_index =
			|map: &mut HashMap<char, HashSet<usize>>, index: usize, text: &str| {
				use kana::CharKind;
				// Iterate the characters on the text and append the entry index
				// to the respective character index sets.
				for chr in text.chars() {
					let indexable = match kana::get_kind(chr) {
						CharKind::Hiragana
						| CharKind::Katakana
						| CharKind::KatakanaHalfWidth
						| CharKind::Kanji
						| CharKind::JapaneseSymbol => true,
						_ => false,
					};
					if indexable {
						let entry = map.entry(chr).or_default();
						entry.insert(index);
					}
				}
			};

		let mut terms_index: HashMap<usize, HashSet<usize>> = HashMap::new();
		let mut search_index: HashMap<usize, HashSet<usize>> = HashMap::new();
		for (entry_index, term) in dictionary.iter().enumerate() {
			let entry_index = entry_index + 1;
			let entry_expr = terms_index.entry(term.expr).or_default();
			entry_expr.insert(entry_index);
			append_char_index(&mut chars_index, entry_index, &terms[term.expr - 1]);
			if term.read != 0 {
				let entry_read = terms_index.entry(term.read).or_default();
				entry_read.insert(entry_index);
				append_char_index(&mut chars_index, entry_index, &terms[term.read - 1]);
			}
			if term.look != 0 {
				let entry_look = search_index.entry(term.look).or_default();
				entry_look.insert(entry_index);
			}
		}

		let build_index = |input: HashMap<usize, HashSet<usize>>| {
			let mut index: Vec<_> = input
				.into_iter()
				.map(|(term_index, entries)| {
					let mut entries: Vec<_> = entries.into_iter().collect();
					entries.sort();
					(term_index, entries)
				})
				.collect();
			index.sort_by(|(index_a, _), (index_b, _)| index_a.cmp(&index_b));
			let index: Vec<_> = index.into_iter().map(|x| x.1).collect();
			index
		};

		let terms_index = build_index(terms_index);
		let search_index = build_index(search_index);

		// Flatten the character index and sort it by set size.
		let mut chars_index: Vec<_> = chars_index
			.into_iter()
			.map(|(chr, indexes)| {
				let mut indexes: Vec<_> = indexes.into_iter().collect();
				indexes.sort();
				(chr, indexes)
			})
			.collect();
		chars_index.sort_by(|(_, a), (_, b)| a.len().cmp(&b.len()));

		println!("... - compiled dictionary indexes ({:?})", start.elapsed());

		let avg: usize = chars_index.iter().map(|x| x.1.len()).sum();
		let avg = avg / chars_index.len();
		println!(
			"... - Char index: {} lines, {} average, {} maximum",
			chars_index.len(),
			avg,
			chars_index[chars_index.len() - 1].1.len()
		);

		//
		// Reverse index:
		//

		// Build a list of indexes from `terms` sorted by the reverse string
		// order.
		let build_reverse_list = |terms: &Vec<&str>| {
			// Compile a list with the original indexes and the reversed string.
			let mut reverse_list: Vec<_> = terms
				.iter()
				.enumerate()
				.map(|(index, term)| {
					// Split the string into graphemes (`true` refers to extended
					// grapheme clusters, the recommended) and reverse.
					let reversed: String = term.graphemes(true).rev().collect();
					(index + 1, reversed)
				})
				.collect();
			// Sort the list by the reversed string.
			reverse_list.sort_by(|(_, a), (_, b)| a.cmp(&b));
			// Take only the indexes.
			let reverse_list: Vec<_> = reverse_list.into_iter().map(|x| x.0).collect();
			reverse_list
		};

		let start = Instant::now();
		let terms_index_reverse = build_reverse_list(&terms);
		let search_index_reverse = build_reverse_list(&search);
		println!("... - build reverse indexes ({:?})", start.elapsed());

		//
		// English index:
		//

		let start = Instant::now();
		for (index, term) in dictionary.iter().enumerate() {
			for def in term.defs.iter() {
				for it in def.text.iter() {
					for sub in RE_SPLIT_ENGLISH.split(&glossary[*it - 1]) {
						if sub.len() > 0 && !kana::is_japanese(sub.chars().next().unwrap(), true) {
							let key = deunicode::deunicode(sub).to_lowercase();
							let entry = english.entry(key).or_default();
							entry.insert(index + 1);
						}
					}
				}
			}
		}

		for key in english.keys() {
			if !RE_ENGLISH_VALIDATE.is_match(key) {
				println!("[WARN] invalid english key: {}", key);
			}
		}

		let mut english = english.iter().collect::<Vec<_>>();
		english.sort_by(|a, b| a.0.cmp(&b.0));
		println!("... - compiled english index ({:?})", start.elapsed());

		//====================================================================//
		// Output
		//====================================================================//

		let mut data_dir = std::env::current_dir().unwrap();
		data_dir.push(output_directory);
		let data_dir = std::fs::canonicalize(data_dir).unwrap();
		std::fs::create_dir_all(&data_dir)?;

		//--------------------------------------------------------------------//
		// tags.json
		//--------------------------------------------------------------------//

		println!("... writing tags.json");
		let mut tags_path = data_dir.clone();
		tags_path.push("tags.json");
		let tags_file = BufWriter::new(fs::File::create(tags_path)?);
		serde_json::to_writer_pretty(tags_file, &tags)?;

		//--------------------------------------------------------------------//
		// kanji.json
		//--------------------------------------------------------------------//

		let mut kanji_path = data_dir.clone();
		kanji_path.push("kanji.json");

		let kanji_file = BufWriter::new(fs::File::create(kanji_path)?);
		serde_json::to_writer_pretty(kanji_file, &kanji)?;

		//--------------------------------------------------------------------//
		// Dictionary data
		//--------------------------------------------------------------------//

		println!("... writing dictionary ({} entries)", dictionary.len());

		let mut dictionary_main_path = data_dir.clone();
		dictionary_main_path.push("dictionary_main.txt");

		let mut dictionary_data_path = data_dir.clone();
		dictionary_data_path.push("dictionary_data.txt");

		let mut dictionary_main_file = BufWriter::new(fs::File::create(dictionary_main_path)?);
		let mut dictionary_data_file = BufWriter::new(fs::File::create(dictionary_data_path)?);
		for it in dictionary.iter() {
			write!(
				dictionary_main_file,
				"{},{},{},{}\n",
				it.expr, it.read, it.look, it.freq,
			)?;
			write!(
				dictionary_data_file,
				"{}\n",
				serde_json::to_string(&it.defs)?
			)?;
		}

		//--------------------------------------------------------------------//
		// sources.json
		//--------------------------------------------------------------------//

		println!("... writing sources.txt");
		let mut sources_path = data_dir.clone();
		sources_path.push("sources.txt");
		let mut sources_file = BufWriter::new(fs::File::create(sources_path)?);
		for it in sources.iter() {
			write!(sources_file, "{}\n", it)?;
		}

		//--------------------------------------------------------------------//
		// terms.txt
		//--------------------------------------------------------------------//

		println!("... writing terms.txt");
		let mut terms_path = data_dir.clone();
		terms_path.push("terms.txt");

		let mut terms_file = BufWriter::new(fs::File::create(terms_path)?);
		for it in terms.iter() {
			write!(terms_file, "{}\n", it)?;
		}

		//--------------------------------------------------------------------//
		// glossary.txt
		//--------------------------------------------------------------------//

		println!("... writing glossary.txt");
		let mut glossary_path = data_dir.clone();
		glossary_path.push("glossary.txt");

		let mut glossary_file = BufWriter::new(fs::File::create(glossary_path)?);
		for it in glossary.iter() {
			write!(glossary_file, "{}\n", it)?;
		}

		//--------------------------------------------------------------------//
		// english.txt
		//--------------------------------------------------------------------//

		println!("... writing english.txt");
		let mut english_path = data_dir.clone();
		english_path.push("english.txt");

		let mut english_file = BufWriter::new(fs::File::create(english_path)?);
		for (word, indexes) in english.iter() {
			let mut indexes = indexes.iter().collect::<Vec<_>>();
			indexes.sort();
			write!(english_file, "{}", word)?;
			for it in indexes {
				write!(english_file, ",{}", it)?;
			}
			write!(english_file, "\n")?;
		}

		//--------------------------------------------------------------------//
		// search.txt
		//--------------------------------------------------------------------//

		println!("... writing search.txt");
		let mut search_path = data_dir.clone();
		search_path.push("search.txt");

		let mut search_file = BufWriter::new(fs::File::create(search_path)?);
		for it in search.iter() {
			write!(search_file, "{}\n", it)?;
		}

		//--------------------------------------------------------------------//
		// Indexes
		//--------------------------------------------------------------------//

		let write_index =
			|outdir: std::path::PathBuf, name: &str, index_data: Vec<Vec<usize>>| -> Result<()> {
				let mut index_path = outdir;
				index_path.push(name);

				let mut output = BufWriter::new(fs::File::create(index_path)?);
				println!("... writing {}", name);
				for indexes in index_data {
					for (n, it) in indexes.iter().enumerate() {
						write!(output, "{}{}", if n > 0 { "," } else { "" }, it)?;
					}
					write!(output, "\n")?;
				}
				Ok(())
			};

		write_index(data_dir.clone(), "terms_index.txt", terms_index)?;
		write_index(data_dir.clone(), "search_index.txt", search_index)?;

		//--------------------------------------------------------------------//
		// chars_index.txt
		//--------------------------------------------------------------------//

		println!("... writing chars_index.txt");
		let mut chars_index_path = data_dir.clone();
		chars_index_path.push("chars_index.txt");

		let mut chars_index_file = BufWriter::new(fs::File::create(chars_index_path)?);
		for (chr, indexes) in chars_index {
			write!(chars_index_file, "{},{}", chr, indexes.len())?;
			output_csv_range(&mut chars_index_file, indexes)?;
			write!(chars_index_file, "\n")?;
		}

		//--------------------------------------------------------------------//
		// Reverse indexes
		//--------------------------------------------------------------------//

		let write_reverse_index =
			|outdir: std::path::PathBuf, name: &str, index_data: Vec<usize>| -> Result<()> {
				let mut index_path = outdir;
				index_path.push(name);

				let mut output = BufWriter::new(fs::File::create(index_path)?);
				println!("... writing {}", name);
				for it in index_data {
					write!(output, "{}\n", it)?;
				}
				Ok(())
			};

		write_reverse_index(
			data_dir.clone(),
			"terms_index_reverse.txt",
			terms_index_reverse,
		)?;
		write_reverse_index(
			data_dir.clone(),
			"search_index_reverse.txt",
			search_index_reverse,
		)?;

		Ok(())
	}

	fn append_tag(&mut self, tag: Tag) {
		if let Some(mut old_tag) = self.tag_map.get_mut(&tag.name) {
			if tag.notes.len() > 0 && tag.notes != old_tag.notes {
				if old_tag.notes.len() > 0 {
					old_tag.notes = format!("{} / {}", old_tag.notes, tag.notes);
				} else {
					old_tag.notes = tag.notes;
				}
			}
			if tag.category != "" && tag.category != old_tag.category {
				if old_tag.category != "" {
					eprintln!(
						"WARNING: overridden category of tag `{}` (was `{}`, with `{}`)",
						tag.name, old_tag.category, tag.category,
					)
				}
				old_tag.category = tag.category;
			}
		} else {
			self.tag_map.insert(tag.name.clone(), tag);
		}
	}

	fn map_tags(&mut self, tags: Vec<String>) {
		for name in tags {
			self.append_tag(Tag {
				name: name,
				category: String::new(),
				order: 0,
				notes: String::new(),
			})
		}
	}
}

#[derive(Serialize)]
struct TermData {
	expr: usize,
	read: usize,
	look: usize,
	freq: u32,
	defs: Vec<DefinitionData>,
}

#[derive(Serialize)]
struct DefinitionData {
	tags_term: Vec<usize>,
	tags_text: Vec<usize>,
	text: Vec<usize>,
	rules: Vec<usize>,
	source: usize,
	score: i32,
}

#[derive(Serialize)]
struct TagData {
	index: usize,
	name: String,
	category: String,
	notes: String,
	order: i32,
}

fn output_csv_range(mut w: impl std::io::Write, ls: Vec<usize>) -> Result<()> {
	let mut cur = 0;
	while cur < ls.len() {
		let sta = ls[cur];
		let pos = cur;
		while cur < ls.len() - 1 && ls[cur + 1] == sta + (cur - pos) + 1 {
			cur += 1;
		}
		let end = ls[cur];
		cur += 1;
		if end > sta {
			write!(w, ",{}-{}", sta, end)?;
		} else {
			write!(w, ",{}", sta)?;
		}
	}
	Ok(())
}

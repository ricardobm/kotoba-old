#![feature(vec_into_raw_parts)]

extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate zip;

use serde::Deserialize;

use std::fs;
use std::io::Read;
use std::io::Write;
use std::time::Instant;

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use zip::{ZipArchive, ZipWriter};

mod raw;
use raw::RawUint32;

/// Directory with files imported by `dict-import`, relative to `Cargo.toml`.
const IMPORTED_DATA_DIRECTORY: &'static str = "../build/imported";

/// Directory where to build the database files.
const OUTPUT_DATA_DIRECTORY: &'static str = "../data/database";

fn main() {
	generate();
}

#[allow(dead_code)]
fn load_and_test() {
	let mut db_dir = std::env::current_dir().unwrap();
	db_dir.push(OUTPUT_DATA_DIRECTORY);

	let start = Instant::now();

	let mut chars_path = db_dir.clone();
	chars_path.push("chars.zip");

	let chars = File::open(chars_path).unwrap();
	let chars = BufReader::new(chars);
	let mut chars = ZipArchive::new(chars).unwrap();
	println!("Loaded zip file in {:?}", start.elapsed());

	let start = Instant::now();

	let q1 = query_chars(&mut chars, '\u{3046}');
	let q2 = query_chars(&mut chars, '\u{3093}');
	let q3 = query_chars(&mut chars, '\u{3057}');
	let q4 = query_chars(&mut chars, '\u{3044}');

	println!("Query finished in {:?}", start.elapsed());

	let start = Instant::now();
	let all = intersect(vec![&q1, &q2, &q3, &q4]);

	let elapsed = start.elapsed();
	println!("Found {} results", all.len());
	println!("Merge finished in {:?}", elapsed);
}

fn intersect(mut ls: Vec<&[u32]>) -> Vec<u32> {
	let mut out = Vec::new();
	if ls.len() == 0 {
		return out;
	} else if ls.len() == 1 {
		out.extend_from_slice(ls[0]);
		return out;
	}

	ls.sort_by(|a, b| a.len().cmp(&b.len()));
	for &next in ls[0].iter() {
		let mut included = true;
		for i in 1..ls.len() {
			let mut cur = ls[i];
			match cur.binary_search(&next) {
				Ok(index) => {
					cur = &cur[index + 1..];
				}
				Err(index) => {
					included = false;
					cur = &cur[index..];
				}
			}
			ls[i] = cur;
		}

		if included {
			out.push(next);
		}
	}
	out
}

fn query_chars<T: std::io::Read + std::io::Seek>(file: &mut ZipArchive<T>, chr: char) -> Vec<u32> {
	let filename = format!("{:06X}", chr as u32);

	println!("- Loading {}...", filename);
	let start = Instant::now();
	let mut out = Vec::new();
	if let Ok(mut file) = file.by_name(filename.as_str()) {
		let mut buffer: Vec<u8> = Vec::with_capacity(file.size() as usize);
		file.read_to_end(&mut buffer).expect("zip read failed");
		println!(
			"- Loaded buffer with size {} in {:?}",
			buffer.len(),
			start.elapsed()
		);

		let start = Instant::now();

		let (ptr, len, cap) = buffer.into_raw_parts();
		let indexes = unsafe {
			let ptr = ptr as *mut RawUint32;
			let len = len / 4;
			let cap = cap / 4;
			Vec::from_raw_parts(ptr, len, cap)
		};

		out.reserve(indexes.len());

		let mut indexes = indexes.into_iter().map(|x| Into::<u32>::into(x));
		while let Some(index) = indexes.next() {
			if index & 0x8000_0000 != 0 {
				let sta = index & 0x0FFF_FFFF;
				let end = indexes.next().unwrap();
				assert!(sta < end);
				for index in sta..=end {
					out.push(index);
				}
			} else {
				out.push(index);
			}
		}
		println!("- Loaded {} indexes in {:?}", out.len(), start.elapsed());
	}

	out
}

#[allow(dead_code)]
fn generate() {
	let mut input_dir = std::env::current_dir().unwrap();
	input_dir.push(IMPORTED_DATA_DIRECTORY);
	let input_dir = fs::canonicalize(input_dir).unwrap();

	let input_dir_str = input_dir.to_string_lossy();
	match fs::metadata(&input_dir) {
		Ok(md) if md.is_dir() => {
			println!("\nLoading data from {:}...", input_dir_str);
		}
		_ => {
			eprintln!("\nERROR: data directory not found at {:}\n", input_dir_str);
			std::process::exit(1);
		}
	};

	let mut output_dir = std::env::current_dir().unwrap();
	output_dir.push(OUTPUT_DATA_DIRECTORY);
	fs::create_dir_all(&output_dir).unwrap();

	let output_dir = std::fs::canonicalize(output_dir).unwrap();
	println!("Generating data to {}...", output_dir.to_string_lossy());

	generate_dict(input_dir.clone(), output_dir.clone());
	generate_text(input_dir.clone(), output_dir.clone());
	generate_meta(input_dir.clone(), output_dir.clone());
	generate_kanji(input_dir.clone(), output_dir.clone());
	generate_chars(input_dir.clone(), output_dir.clone());
}

#[derive(Deserialize)]
struct DictDefinition {
	source: u32,
	score: i32,
	text: Vec<u32>,
	rules: Vec<u32>,
	tags_term: Vec<u32>,
	tags_text: Vec<u32>,
}

fn generate_dict(input_dir: PathBuf, output_dir: PathBuf) {
	let start = Instant::now();

	let mut dict_output = output_dir;
	dict_output.push("dict.zip");

	let dict = BufWriter::new(File::create(dict_output).unwrap());
	let mut dict = ZipWriter::new(dict);

	let mut dict_index = Vec::new();

	let mut dict_main_path = input_dir.clone();
	dict_main_path.push("dictionary_main.txt");
	let dict_main_text = fs::read_to_string(dict_main_path).unwrap();
	for line in dict_main_text.lines() {
		let fields = line
			.split(',')
			.map(|x| RawUint32::from(x.parse::<u32>().unwrap()));
		dict_index.extend(fields);
	}

	dict.start_file("index", Default::default()).unwrap();
	write_zip(&mut dict, unsafe { vec_bytes(&dict_index) });

	let mut dict_data_path = input_dir.clone();
	dict_data_path.push("dictionary_data.txt");
	let dict_data_text = fs::read_to_string(dict_data_path).unwrap();

	let mut cur_file = -1;
	let mut cur_file_data: Vec<RawUint32> = Vec::new();
	let mut cur_file_index: Vec<RawUint32> = Vec::new();

	fn push_file(
		zip: &mut ZipWriter<BufWriter<File>>,
		number: i32,
		data: Vec<RawUint32>,
		index: Vec<RawUint32>,
	) {
		if number >= 0 {
			zip.start_file(format!("{:04}", number), Default::default())
				.unwrap();
			write_zip(zip, unsafe {
				vec_bytes(&vec![
					RawUint32::from(index.len()),
					RawUint32::from(data.len()),
				])
			});
			write_zip(zip, unsafe { vec_bytes(&index) });
			write_zip(zip, unsafe { vec_bytes(&data) });
		}
	}

	for (index, line) in dict_data_text.lines().enumerate() {
		let file_num = (index / 1000) as i32;
		if file_num != cur_file {
			push_file(&mut dict, cur_file, cur_file_data, cur_file_index);
			cur_file_data = Vec::new();
			cur_file_index = Vec::new();
			cur_file = file_num;
		}

		let mut definitions: Vec<DictDefinition> = serde_json::from_str(line).unwrap();
		definitions.sort_by_key(|x| -x.score);

		fn push_list(out: &mut Vec<RawUint32>, data: Vec<u32>) {
			out.push(data.len().into());
			out.extend(data.into_iter().map(|x| RawUint32::from(x)));
		}

		let mut data: Vec<RawUint32> = Vec::new();
		data.push(definitions.len().into());
		for entry in definitions {
			data.push(entry.source.into());
			push_list(&mut data, entry.text);
			push_list(&mut data, entry.rules);
			push_list(&mut data, entry.tags_term);
			push_list(&mut data, entry.tags_text);
		}

		cur_file_index.push(cur_file_data.len().into());
		cur_file_index.push(data.len().into());
		cur_file_data.append(&mut data);
	}

	push_file(&mut dict, cur_file, cur_file_data, cur_file_index);

	println!("Wrote dict.zip in {:?}", start.elapsed());
}

fn generate_text(input_dir: PathBuf, output_dir: PathBuf) {
	let start = Instant::now();

	let mut text_output = output_dir;
	text_output.push("text.zip");

	let text = BufWriter::new(File::create(text_output).unwrap());
	let mut text = ZipWriter::new(text);

	let mut glossary_input_path = input_dir.clone();
	glossary_input_path.push("glossary.txt");
	let glossary = fs::read_to_string(glossary_input_path).unwrap();
	text.start_file("glossary", Default::default()).unwrap();
	generate_text_data_file(&mut text, &glossary);

	let mut english_input_path = input_dir.clone();
	english_input_path.push("english.txt");
	let english = fs::read_to_string(english_input_path).unwrap();

	let mut english_index = Vec::new();
	let mut english_data = Vec::new();
	let mut english_list = Vec::new();
	for line in english.lines() {
		let mut fields = line.split(',');
		let word = fields.next().unwrap();
		let (text_offset, text_length) = (english_data.len(), word.len());
		english_data.extend_from_slice(word.as_bytes());

		let mut list: Vec<_> = fields
			.map(|x| RawUint32::from(x.parse::<u32>().unwrap()))
			.collect();
		let (list_offset, list_length) = (english_list.len(), list.len());
		english_list.append(&mut list);

		english_index.push((
			RawUint32::from(text_offset),
			RawUint32::from(text_length),
			RawUint32::from(list_offset),
			RawUint32::from(list_length),
		));
	}

	let english_index_bytes = unsafe { vec_bytes(&english_index) };
	let english_list_bytes = unsafe { vec_bytes(&english_list) };
	text.start_file("glossary_index", Default::default())
		.unwrap();
	write_zip(&mut text, unsafe {
		vec_bytes(&vec![
			RawUint32::from(english_index_bytes.len()),
			RawUint32::from(english_data.len()),
			RawUint32::from(english_list_bytes.len()),
		])
	});
	write_zip(&mut text, english_index_bytes);
	write_zip(&mut text, &english_data);
	write_zip(&mut text, &english_list_bytes);

	generate_text_index(&mut text, input_dir.clone(), "terms");
	generate_text_index(&mut text, input_dir.clone(), "search");

	println!("Wrote text.zip in {:?}", start.elapsed());
}

fn generate_text_data_file(zip: &mut ZipWriter<BufWriter<File>>, text: &str) {
	let mut count = 0 as u32;
	let mut index = Vec::new();
	let mut data = Vec::new();
	for line in text.lines() {
		let (offset, length) = (data.len(), line.len());
		let offset = RawUint32::from(offset);
		let length = RawUint32::from(length);
		count += 1;
		index.push((offset, length));
		data.extend_from_slice(line.as_bytes());
	}

	let count = RawUint32::from(count);
	write_zip(zip, &count.bytes());
	write_zip(zip, unsafe { vec_bytes(&index) });
	write_zip(zip, &data);
}

fn generate_text_index(zip: &mut ZipWriter<BufWriter<File>>, input_dir: PathBuf, name: &str) {
	let mut text_input_path = input_dir.clone();
	text_input_path.push(format!("{}.txt", name));
	let text = fs::read_to_string(text_input_path).unwrap();

	zip.start_file(format!("{}_text", name), Default::default())
		.unwrap();
	generate_text_data_file(zip, &text);

	let mut index_input_path = input_dir.clone();
	index_input_path.push(format!("{}_index.txt", name));
	let index = fs::read_to_string(index_input_path).unwrap();

	let mut index_count = 0 as u32;
	let mut index_index = Vec::new();
	let mut index_data = Vec::new();
	for line in index.lines() {
		let mut items: Vec<RawUint32> = line
			.split(',')
			.map(|x| x.parse::<u32>().unwrap().into())
			.collect();
		index_count += 1;

		let (offset, length) = (index_data.len(), items.len());
		let offset = RawUint32::from(offset);
		let length = RawUint32::from(length);
		index_index.push((offset, length));
		index_data.append(&mut items);
	}

	let index_count = RawUint32::from(index_count);
	zip.start_file(format!("{}_index", name), Default::default())
		.unwrap();
	write_zip(zip, &index_count.bytes());
	write_zip(zip, unsafe { vec_bytes(&index_index) });
	write_zip(zip, unsafe { vec_bytes(&index_data) });

	let mut reverse_input_path = input_dir.clone();
	reverse_input_path.push(format!("{}_index_reverse.txt", name));
	let reverse = fs::read_to_string(reverse_input_path).unwrap();

	let mut reverse_data = vec![RawUint32::from(0 as u32)];
	for line in reverse.lines() {
		let item: RawUint32 = line.parse::<u32>().unwrap().into();
		reverse_data.push(item);
	}

	zip.start_file(format!("{}_reverse", name), Default::default())
		.unwrap();
	write_zip(zip, unsafe { vec_bytes(&reverse_data) });
}

fn generate_kanji(input_dir: PathBuf, output_dir: PathBuf) {
	let start = Instant::now();

	let mut kanji_output = output_dir;
	kanji_output.push("kanji.zip");

	let kanji = BufWriter::new(File::create(kanji_output).unwrap());
	let mut kanji = ZipWriter::new(kanji);

	let mut kanji_input_path = input_dir;
	kanji_input_path.push("kanji.json");

	let kanji_input = fs::read_to_string(kanji_input_path).unwrap();
	kanji.start_file("kanji.json", Default::default()).unwrap();

	let bytes = kanji_input.as_bytes();
	write_zip(&mut kanji, &bytes);

	kanji.finish().unwrap();
	println!("Wrote kanji.zip in {:?}", start.elapsed());
}

fn generate_meta(input_dir: PathBuf, output_dir: PathBuf) {
	let start = Instant::now();

	let mut meta_output = output_dir;
	meta_output.push("meta.zip");

	let meta = BufWriter::new(File::create(meta_output).unwrap());
	let mut meta = ZipWriter::new(meta);

	let mut tags_path = input_dir.clone();
	tags_path.push("tags.json");

	meta.start_file("tags.json", Default::default()).unwrap();
	write_zip(&mut meta, &fs::read(tags_path).unwrap());

	let mut sources_path = input_dir.clone();
	sources_path.push("sources.txt");

	meta.start_file("sources.txt", Default::default()).unwrap();
	write_zip(&mut meta, &fs::read(&sources_path).unwrap());

	meta.finish().unwrap();
	println!("Wrote meta.zip in {:?}", start.elapsed());
}

fn generate_chars(input_dir: PathBuf, output_dir: PathBuf) {
	let start = Instant::now();

	let mut chars_output = output_dir;
	chars_output.push("chars.zip");

	let chars = BufWriter::new(File::create(chars_output).unwrap());
	let mut chars = ZipWriter::new(chars);

	let mut chars_index_path = input_dir;
	chars_index_path.push("chars_index.txt");

	let chars_index = fs::read_to_string(chars_index_path).unwrap();
	for line in chars_index.lines() {
		let line = line.trim();
		if line.len() > 0 {
			let mut fields = line.split(',');

			let character = fields.next().unwrap();
			let character: Vec<_> = character.chars().collect();
			assert!(character.len() == 1);
			let character = character[0];

			let count = fields.next().unwrap().parse::<usize>().unwrap();
			let mut indexes = Vec::with_capacity(count);
			for index in fields {
				let range: Vec<&str> = index.split('-').collect();
				let sta: u32 = range[0].parse().unwrap();
				let end = if range.len() == 2 {
					range[1].parse().unwrap()
				} else {
					sta
				};

				if end > sta {
					assert!(sta <= 0x0FFF_FFFF);
					indexes.push(RawUint32::from(sta + 0x8000_0000));
					indexes.push(RawUint32::from(end));
				} else {
					indexes.push(RawUint32::from(sta));
				}
			}

			let filename = format!("{:06X}", character as u32);
			chars
				.start_file(&filename, zip::write::FileOptions::default())
				.unwrap();

			let bytes = unsafe { vec_bytes(&indexes) };
			write_zip(&mut chars, &bytes);
		}
	}

	chars.finish().unwrap();

	println!("Wrote chars.zip in {:?}", start.elapsed());
}

fn write_zip(output: &mut ZipWriter<BufWriter<File>>, mut bytes: &[u8]) {
	while bytes.len() > 0 {
		let written = output.write(bytes).unwrap();
		assert!(written > 0);
		bytes = &bytes[written..];
	}
}

#[inline]
unsafe fn vec_bytes<T: Sized>(value: &Vec<T>) -> &[u8] {
	let start = value.as_slice().as_ptr();
	std::slice::from_raw_parts(start as *const u8, std::mem::size_of::<T>() * value.len())
}

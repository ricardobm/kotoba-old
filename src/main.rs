#![feature(proc_macro_hygiene, decl_macro)]
#![feature(duration_float)]

extern crate itertools;

#[macro_use]
extern crate serde;
extern crate serde_json;
extern crate serde_tuple;

extern crate bincode;

extern crate rand;
extern crate regex;

extern crate zip;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

extern crate wana_kana;

use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time;

use rand::seq::SliceRandom;
use rand::thread_rng;

const FROM_ZIP: bool = false;
const DUMP_WORD_SAMPLE: bool = false;

mod config;

mod dict;
use dict::import;
use dict::Dict;

mod server;

mod japanese;

fn main() {
	std::process::exit(run());
}

fn run() -> i32 {
	let data_dir = config::data_directory();
	println!("\nUser data directory is {}", data_dir.to_string_lossy());

	let mut dict_path = PathBuf::from(&data_dir);
	dict_path.push("dict");

	let mut ok_file = PathBuf::from(&dict_path);
	ok_file.push("ok.dat");

	if let Err(_) = fs::metadata(&ok_file) {
		println!("\nDictionary data not found, importing...");

		let dict = import_entries(data_dir);

		println!("\nSaving dictionary data to {}...", dict_path.to_string_lossy());
		let start = time::Instant::now();
		dict.save(&dict_path).unwrap();
		File::create(&ok_file).unwrap();
		let duration = start.elapsed();
		println!("\n#\n# Serialization took {:?}\n#", duration);
	}

	println!("\nLoading dictionary data from {}...", dict_path.to_string_lossy());

	let start = time::Instant::now();
	let mut dict = Dict::load(&dict_path).unwrap();
	println!("Loaded {} entries in {:?}", dict.count(), start.elapsed());

	let start = time::Instant::now();
	dict.rebuild_index();
	println!("Indexed in {:?}", start.elapsed());

	if DUMP_WORD_SAMPLE {
		let mut rng = thread_rng();
		println!("\n##\n## ENTRIES ##\n##");
		dict.shuffle(&mut rng);
		for it in dict.entries().into_iter().take(10) {
			println!("\n{}", it);
		}
	}

	println!();

	server::launch(japanese::Dictionary::new(dict));

	0
}

fn import_entries(user_data: &Path) -> Dict {
	let mut import_path = PathBuf::from(&user_data);
	import_path.push(if FROM_ZIP { "import" } else { "import-files" });

	let start = time::Instant::now();
	let imported = import::import_from(&import_path).unwrap();
	let duration = start.elapsed();
	println!("\nLoaded {} import files in {:?}", imported.len(), duration);

	let mut builder = dict::DictBuilder::new();

	let mut rng = thread_rng();
	for mut it in imported {
		{
			it.append_to(&mut builder);
		}

		println!("\n\n{}", it);

		if DUMP_WORD_SAMPLE {
			it.terms.as_mut_slice().shuffle(&mut rng);
			it.kanjis.as_mut_slice().shuffle(&mut rng);
			it.meta_terms.as_mut_slice().shuffle(&mut rng);
			it.meta_kanjis.as_mut_slice().shuffle(&mut rng);

			if it.tags.len() > 0 {
				println!("\n## Tags ##\n");
				for tag in it.tags {
					println!("- {}", tag);
				}
			}

			if it.terms.len() > 0 {
				println!("\n## Terms ##\n");
				for term in it.terms.iter().take(3) {
					println!("\n{}", term);
				}
			}

			if it.kanjis.len() > 0 {
				println!("\n## Kanjis ##\n");
				for kanji in it.kanjis.iter().take(3) {
					println!("\n{}", kanji);
				}
			}

			if it.meta_terms.len() > 0 {
				println!("\n## Meta (Terms) ##\n");
				for meta in it.meta_terms.iter().take(10) {
					println!("- {}", meta);
				}
			}

			if it.meta_kanjis.len() > 0 {
				println!("\n## Meta (Kanjis) ##\n");
				for meta in it.meta_kanjis.iter().take(10) {
					println!("- {}", meta);
				}
			}
		}
	}

	let dict = builder.build();

	println!(
		"\n#\n# Imported {} total entries in {:?}\n#",
		dict.count(),
		start.elapsed()
	);
	dict
}

#[cfg(test)]
mod tests {
	#[test]
	fn should_succeed() {
		assert_eq!(42, 42);
	}
}

#[macro_use]
extern crate serde;
extern crate serde_json;

extern crate rand;
extern crate regex;

#[macro_use]
extern crate lazy_static;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::time;

use rand::seq::SliceRandom;
use rand::thread_rng;

const DATA_DIR: &str = "hongo-data";

mod dict;
use dict::import::*;

fn main() {
	std::process::exit(run());
}

fn run() -> i32 {
	// Find the User data directory
	let mut cur_dir = env::current_dir().unwrap();
	let user_data = loop {
		let mut data_path = PathBuf::from(&cur_dir);
		data_path.push(DATA_DIR);

		let mut test_path = PathBuf::from(&data_path);
		test_path.push("import");
		if let Ok(md) = fs::metadata(&test_path) {
			if md.is_dir() {
				break Some(data_path);
			}
		}

		if let Some(dir) = cur_dir.parent() {
			cur_dir = dir.to_path_buf();
		} else {
			break None;
		}
	};

	let user_data = match user_data {
		None => {
			eprintln!("\nError: could not find the user directory");
			return 1;
		}
		Some(user_data) => {
			println!("\nUser data directory is {:}\n", user_data.to_str().unwrap());
			user_data
		}
	};

	let mut import_path = PathBuf::from(user_data);
	import_path.push("import-files");

	let start = time::Instant::now();
	let imported = import_from(&import_path).unwrap();
	let duration = start.elapsed();
	println!("\nImported {} entries in {:?}", imported.len(), duration);

	let mut all_entries = Vec::new();
	let mut rng = thread_rng();
	for mut it in imported {
		println!("\n\n{}", it);
		it.terms.as_mut_slice().shuffle(&mut rng);
		it.kanjis.as_mut_slice().shuffle(&mut rng);
		it.meta_terms.as_mut_slice().shuffle(&mut rng);
		it.meta_kanjis.as_mut_slice().shuffle(&mut rng);

		for term in &it.terms {
			all_entries.push(term.to_entry(&it));
		}

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

	println!();

	println!("\n## ENTRIES ##\n");
	all_entries.as_mut_slice().shuffle(&mut rng);
	for it in all_entries.iter().take(10) {
		println!("\n{}", it);
	}

	0
}

#[cfg(test)]
mod tests {
	#[test]
	fn should_succeed() {
		assert_eq!(42, 42);
	}
}

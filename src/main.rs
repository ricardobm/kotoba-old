#[macro_use]
extern crate serde;
extern crate serde_json;

extern crate regex;

#[macro_use]
extern crate lazy_static;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::time;

const DATA_DIR: &str = "hongo-data";

mod import;
use import::*;

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
			println!(
				"\nUser data directory is {:}\n",
				user_data.to_str().unwrap()
			);
			user_data
		}
	};

	let mut import_path = PathBuf::from(user_data);
	import_path.push("import-files");

	let start = time::Instant::now();
	let imported = import_from(&import_path).unwrap();
	let duration = start.elapsed();
	println!("\nImported {} entries in {:?}", imported.len(), duration);

	for it in imported {
		println!(
			"\n=> {} ({}) -- {}",
			it.title,
			it.revision,
			it.path.to_string_lossy()
		);
		println!("\n## Terms: {}", it.terms.len());
		if it.terms.len() > 0 {
			println!("\nTop 10:\n");
			for term in it.terms.iter().take(10) {
				println!("-> {} 「{}」", term.expression, term.reading);
				if term.term_tags.len() > 0 {
					println!("    (term: {})", term.term_tags.join(", "));
				}
				if term.definition_tags.len() > 0 {
					println!("    (definition: {})", term.definition_tags.join(", "));
				}
				println!("    {}", term.glossary.join("; "));
			}
		}
	}

	println!();

	0
}

#[cfg(test)]
mod tests {
	#[test]
	fn should_succeed() {
		assert_eq!(42, 42);
	}
}

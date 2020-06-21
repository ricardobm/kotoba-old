extern crate deunicode;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate unicase;
extern crate unicode_normalization;
extern crate unicode_segmentation;
extern crate zip;

#[macro_use]
extern crate lazy_static;

extern crate kana;

use std::fs;
use unicase::UniCase;

mod dict;
mod import;
mod writer;

use import::import_file;
use writer::Writer;

/// Directory with the data to be imported, relative to `Cargo.toml`.
const IMPORT_DATA_DIRECTORY: &'static str = "data";

/// Output directory for the generated data, relative to `Cargo.toml`.
const OUTPUT_DATA_DIRECTORY: &'static str = "output";

fn main() {
	let start = std::time::Instant::now();

	let mut data_dir = std::env::current_dir().unwrap();
	data_dir.push(IMPORT_DATA_DIRECTORY);

	let data_dir_str = data_dir.to_string_lossy();
	let data_dir = match fs::metadata(&data_dir) {
		Ok(md) if md.is_dir() => {
			println!("\nImporting from {:}...", data_dir_str);
			data_dir
		}
		_ => {
			eprintln!("\nERROR: data directory not found at {:}\n", data_dir_str);
			std::process::exit(1);
		}
	};

	match import(data_dir) {
		Ok(_) => {
			println!("\nImporting finished after {:?}\n", start.elapsed());
		}
		Err(err) => {
			eprintln!("\nERROR: import failed: {:}\n", err);
			std::process::exit(2);
		}
	}
}

fn import<P: AsRef<std::path::Path>>(import_dir: P) -> std::io::Result<()> {
	let start = std::time::Instant::now();
	let mut entries = Vec::new();
	for entry in fs::read_dir(import_dir)? {
		let entry = entry?;
		if entry.file_type()?.is_file() {
			let fullpath = entry.path();
			if let Some(ext) = fullpath.extension() {
				let ext = ext.to_string_lossy();
				if UniCase::new(ext) == UniCase::new("zip") {
					entries.push(fullpath);
				}
			}
		}
	}

	println!("Found {} file(s) to import...", entries.len());
	let mut writer = Writer::default();
	for fs in entries {
		let dict = import_file(fs)?;
		writer.append_dict(dict);
	}

	println!("\nImported database (elapsed {:?})", start.elapsed());
	let start = std::time::Instant::now();
	println!("\nExporting...");
	writer.output(OUTPUT_DATA_DIRECTORY)?;
	println!("... completed in {:?}", start.elapsed());

	Ok(())
}

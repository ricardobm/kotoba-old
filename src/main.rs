#![feature(proc_macro_hygiene, decl_macro)]
#![feature(duration_float)]

#[macro_use]
extern crate serde;
extern crate serde_json;
extern crate serde_tuple;

extern crate bincode;

extern crate fnv;
extern crate itertools;
extern crate rand;
extern crate regex;
extern crate unicode_normalization;

extern crate zip;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

extern crate data_encoding;
extern crate reqwest;
extern crate ring;
extern crate scraper;

extern crate crossbeam;

use std::path::PathBuf;
use std::time;

const FROM_ZIP: bool = false;
const DUMP_WORD_SAMPLE: bool = false;

mod config;
mod db;
mod dict;
mod import;
mod japanese;
mod kana;
mod pronunciation;
mod server;
mod util;

fn main() {
	std::process::exit(run());
}

#[allow(dead_code)]
fn run() -> i32 {
	let data_dir = config::data_directory();
	println!("\nUser data directory is {}", data_dir.to_string_lossy());

	let mut dict_path = PathBuf::from(&data_dir);
	dict_path.push("dict");
	dict_path.push("imported.db");

	let dict_path_str = dict_path.to_string_lossy();

	let start = time::Instant::now();
	let mut db = if let Some(db) = db::Root::load(&dict_path).unwrap() {
		println!(
			"Loaded database from {} in {:.3}s",
			dict_path_str,
			start.elapsed().as_secs_f64()
		);
		db
	} else {
		println!("Database not found in {}!", dict_path_str);

		let import_path = {
			let mut p = PathBuf::from(data_dir);
			p.push(if FROM_ZIP { "import" } else { "import-files" });
			p
		};
		println!("Importing from {}...", import_path.to_string_lossy());

		let mut db = db::Root::new();
		import::from_yomichan(&mut db, import_path).unwrap();
		println!(
			"... imported{} files in {:.3}s",
			if FROM_ZIP { " zip" } else { "" },
			start.elapsed().as_secs_f64()
		);

		let start = time::Instant::now();
		let (kanji_count, terms_count) = db.update_frequency();
		println!(
			"... updated frequency metadata for {} kanji and {} terms in {:.3}s",
			kanji_count,
			terms_count,
			start.elapsed().as_secs_f64()
		);

		db.merge_entries();

		db.update_index(true);
		db.save(&dict_path).unwrap();
		db
	};

	if DUMP_WORD_SAMPLE {
		println!();
		db.dump(10, &mut std::io::stdout()).unwrap();
		println!();
	}

	if db.update_index(false) {
		db.save(&dict_path).unwrap();
	}

	server::launch(server::Data {
		dict:  japanese::Dictionary::new(db),
		audio: pronunciation::JapaneseService::new(&data_dir.join("audio")),
	});

	0
}

#![feature(vec_into_raw_parts)]

#[macro_use]
extern crate lazy_static;

extern crate zip;

mod file_dict;
pub mod file_text;
mod files;
mod raw;

pub use file_dict::Dict;

pub fn version() -> &'static str {
	env!("CARGO_PKG_VERSION")
}

pub fn load() {
	use files::Zip;

	let chars: Zip = files::chars();
	let dict: Zip = files::dict();
	let kanji: Zip = files::kanji();
	let meta: Zip = files::meta();
	let text: Zip = files::text();
	let total_files: usize =
		chars.count() + dict.count() + kanji.count() + meta.count() + text.count();
	println!("Loaded {} files", total_files);

	let dict: Dict = Dict::new(dict).unwrap();
	println!("Loaded {} dictionary entries", dict.count());

	let mut text = text;
	let terms_text = file_text::Text::load_text(&mut text, "terms_text").unwrap();
	println!("Loaded {} terms text entries", terms_text.count());

	for i in 0..10 {
		let base = i * 100;
		println!("Entry {} - {}", base + 0, terms_text.entry(base + 0));
		println!("Entry {} - {}", base + 1, terms_text.entry(base + 1));
		println!("Entry {} - {}", base + 2, terms_text.entry(base + 2));
		println!("Entry {} - {}", base + 3, terms_text.entry(base + 3));
		println!("Entry {} - {}", base + 4, terms_text.entry(base + 4));
		println!("Entry {} - {}", base + 5, terms_text.entry(base + 5));
	}

	let count = terms_text.count();
	for i in (1..=20).rev() {
		println!("Entry {} - {}", count - i, terms_text.entry(count - i));
	}
}

fn bytes(value: usize) -> String {
	if value == 1 {
		String::from("1 byte")
	} else if value < 1024 {
		format!("{} bytes", value)
	} else if value < 1024 * 1024 {
		let kb = (value as f64) / 1024.0;
		format!("{:.2} KB", kb)
	} else {
		let mb = (value as f64) / (1024.0 * 1024.0);
		format!("{:.2} MB", mb)
	}
}

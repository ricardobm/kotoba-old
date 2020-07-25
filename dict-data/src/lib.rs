#[macro_use]
extern crate lazy_static;

mod files;

pub fn version() -> &'static str {
	env!("CARGO_PKG_VERSION")
}

pub fn load() {
	lazy_static! {
		static ref CHARS: &'static [u8] = files::chars();
		static ref DICT: &'static [u8] = files::dict();
		static ref KANJI: &'static [u8] = files::kanji();
		static ref META: &'static [u8] = files::meta();
		static ref TEXT: &'static [u8] = files::text();
	}

	let total = CHARS.len() + DICT.len() + KANJI.len() + META.len() + TEXT.len();
	println!("Loaded {} from 5 files", bytes(total));
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

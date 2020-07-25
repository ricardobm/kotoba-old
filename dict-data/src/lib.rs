#[macro_use]
extern crate lazy_static;

extern crate zip;

mod files;

pub fn version() -> &'static str {
	env!("CARGO_PKG_VERSION")
}

pub fn load() {
	use files::Zip;

	lazy_static! {
		static ref CHARS: Zip = files::chars();
		static ref DICT: Zip = files::dict();
		static ref KANJI: Zip = files::kanji();
		static ref META: Zip = files::meta();
		static ref TEXT: Zip = files::text();
	}

	let total = CHARS.len() + DICT.len() + KANJI.len() + META.len() + TEXT.len();
	println!("Loaded {} files", bytes(total));
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

// The database top directory relative to the Cargo.toml file.
macro_rules! data_dir {
	() => {
		"../data/database"
	};
}

#[allow(unused_macros)]
macro_rules! get_file_data {
	($name: tt) => {
		include_bytes!(concat!("../", data_dir!(), "/", $name));
	};
}

#[allow(unused_macros)]
macro_rules! load_file_data {
	($name: tt) => {
			({
			lazy_static! {
				static ref DATA: Vec<u8> = {
					let mut file_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
					file_path.push(concat!(data_dir!(), "/", $name));
					std::fs::read(file_path).unwrap()
				};
				}
			&DATA
			})
	};
}

#[cfg(not(any(debug_assertions, feature = "no-embed")))]
macro_rules! get_file {
	($name: tt) => {
		get_file_data!($name)
	};
}

#[cfg(any(debug_assertions, feature = "no-embed"))]
macro_rules! get_file {
	($name: tt) => {
		load_file_data!($name)
	};
}

#[inline]
pub fn chars() -> &'static [u8] {
	get_file!("chars.zip")
}

#[inline]
pub fn dict() -> &'static [u8] {
	get_file!("dict.zip")
}

#[inline]
pub fn kanji() -> &'static [u8] {
	get_file!("kanji.zip")
}

#[inline]
pub fn meta() -> &'static [u8] {
	get_file!("meta.zip")
}

#[inline]
pub fn text() -> &'static [u8] {
	get_file!("text.zip")
}

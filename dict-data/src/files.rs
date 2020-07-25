/// Resolves to a literal string with the path to thetop directory for database
/// files relative to `Cargo.toml`.
macro_rules! data_dir {
	() => {
		"../data/database"
	};
}

/// Simple `include_bytes!` wrapper to include a database file from its name.
#[allow(unused_macros)]
macro_rules! get_file_data {
	($name: tt) => {
		include_bytes!(concat!("../", data_dir!(), "/", $name));
	};
}

/// Loads a database file during development.
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

/// Evaluates to a database file data as a `&'static [u8]` from the filename.
#[cfg(not(any(debug_assertions, feature = "no-embed")))]
macro_rules! get_file {
	($name: tt) => {
		get_file_data!($name)
	};
}

/// Evaluates to a database file data as a `&'static [u8]` from the filename.
#[cfg(any(debug_assertions, feature = "no-embed"))]
macro_rules! get_file {
	($name: tt) => {
		load_file_data!($name)
	};
}

/// Returns the contents of `chars.zip`.
#[inline]
pub fn chars() -> Zip {
	zip(get_file!("chars.zip"))
}

/// Returns the contents of `dict.zip`.
#[inline]
pub fn dict() -> Zip {
	zip(get_file!("dict.zip"))
}

/// Returns the contents of `kanji.zip`.
#[inline]
pub fn kanji() -> Zip {
	zip(get_file!("kanji.zip"))
}

/// Returns the contents of `meta.zip`.
#[inline]
pub fn meta() -> Zip {
	zip(get_file!("meta.zip"))
}

/// Returns the contents of `text.zip`.
#[inline]
pub fn text() -> Zip {
	zip(get_file!("text.zip"))
}

/// Simple alias to a ZipArchive.
pub type Zip = zip::ZipArchive<std::io::Cursor<&'static [u8]>>;

/// Creates a Zip from the data bytes.
pub fn zip(data: &'static [u8]) -> Zip {
	let data = std::io::Cursor::new(data);
	Zip::new(data).unwrap()
}

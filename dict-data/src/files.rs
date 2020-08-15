use std::io::{Error, ErrorKind, Read};

use crate::raw::cast_vec;
use crate::raw::RawUint32;

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

type ZipArchive = zip::ZipArchive<std::io::Cursor<&'static [u8]>>;

/// Simple alias to a ZipArchive.
pub struct Zip {
	file: ZipArchive,
}

/// Creates a Zip from the data bytes.
pub fn zip(data: &'static [u8]) -> Zip {
	let data = std::io::Cursor::new(data);
	Zip {
		file: ZipArchive::new(data).unwrap(),
	}
}

impl Zip {
	/// Number of files in the container.
	pub fn count(&self) -> usize {
		self.file.len()
	}

	pub fn open<'a>(&'a mut self, name: &str) -> std::io::Result<ZipFile<'a>> {
		Ok(ZipFile {
			file: self.file.by_name(name)?,
		})
	}

	/// Read a file by name to a typed vector.
	pub fn read_vec<T: Sized>(&mut self, name: &str) -> std::io::Result<Vec<T>> {
		let mut file = self.open(name)?;
		file.read_vec::<T>()
	}
}

pub struct ZipFile<'a> {
	file: zip::read::ZipFile<'a>,
}

impl<'a> ZipFile<'a> {
	/// Read the whole content of the file as a typed vector.
	pub fn read_vec<T: Sized>(&mut self) -> std::io::Result<Vec<T>> {
		let row_size: usize = std::mem::size_of::<T>();

		// Check that the file size is valid for the given row size.
		let file_size = self.file.size() as usize;
		if file_size % row_size != 0 {
			let err = Error::new(
				ErrorKind::InvalidData,
				format!("index file size is invalid ({} bytes)", file_size),
			);
			return Err(err);
		}

		// Allocate a vector large enough to hold all the rows. We need to
		// allocate as T to respect the type alignment.
		let buffer: Vec<T> = Vec::with_capacity(file_size / row_size);

		// Cast the buffer to bytes so that it can be used as a reading buffer.
		let mut buffer: Vec<u8> = unsafe { cast_vec(buffer) };

		// Read the index.
		self.file.read_to_end(&mut buffer)?;

		// Cast the now-filled buffer back to the target type vector.
		let buffer: Vec<T> = unsafe { cast_vec(buffer) };

		Ok(buffer)
	}

	pub fn read_uint(&mut self) -> std::io::Result<RawUint32> {
		let mut buffer = [0; 4];
		self.file.read_exact(&mut buffer)?;
		Ok(RawUint32::from_bytes(&buffer))
	}

	pub fn read_uint_list(&mut self) -> std::io::Result<Vec<RawUint32>> {
		let count = self.read_uint()?;
		let count: usize = count.into();
		self.read_uint_vec(count)
	}

	pub fn read_uint_vec(&mut self, count: usize) -> std::io::Result<Vec<RawUint32>> {
		let mut buffer: Vec<RawUint32> = Vec::with_capacity(count);
		let mut buffer: Vec<u8> = unsafe {
			buffer.set_len(count);
			cast_vec(buffer)
		};

		self.file.read_exact(&mut buffer)?;
		let buffer = unsafe { cast_vec(buffer) };
		Ok(buffer)
	}

	pub fn read_all(&mut self) -> std::io::Result<Vec<u8>> {
		let mut buffer = Vec::new();
		self.file.read_to_end(&mut buffer)?;
		Ok(buffer)
	}
}

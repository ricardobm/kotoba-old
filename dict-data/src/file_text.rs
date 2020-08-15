use crate::files::Zip;
use crate::raw::RawUint32;

/// Wraps a text file.
pub struct Text {
	index: Vec<RawUint32>,
	bytes: Vec<u8>,
}

impl Text {
	pub fn load_text(zip: &mut Zip, name: &str) -> std::io::Result<Text> {
		let mut file = zip.open(name)?;
		let count: usize = file.read_uint()?.into();
		let index = file.read_uint_vec(count * 2)?;
		let bytes = file.read_all()?;
		Ok(Text {
			index: index,
			bytes: bytes,
		})
	}

	pub fn count(&self) -> usize {
		self.index.len() / 2
	}

	pub fn entry(&self, index: usize) -> String {
		let pos = index * 2;
		let offset: usize = self.index[pos + 0].into();
		let length: usize = self.index[pos + 1].into();
		String::from_utf8_lossy(&self.bytes[offset..offset + length]).to_string()
	}
}

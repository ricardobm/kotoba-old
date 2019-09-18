use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use serde_json;

use super::Result;

/// Deserialize a JSON file.
///
/// Returns:
///
/// - `Ok(None)` if the file does not exist;
/// - `Ok(Some(...))` if successful;
/// - `Err(...)` on IO or deserialization errors.
pub fn read_json<P: AsRef<Path>, D: DeserializeOwned>(path: P) -> Result<Option<D>> {
	let path = path.as_ref();
	if path.exists() {
		let input = File::open(path)?;
		let input = BufReader::new(input);
		Ok(Some(serde_json::from_reader(input)?))
	} else {
		Ok(None)
	}
}

/// Write a JSON file.
pub fn write_json<P: AsRef<Path>, T: Serialize>(path: P, value: &T) -> Result<()> {
	let path = path.as_ref();
	let writer = File::create(path)?;
	let writer = BufWriter::new(writer);
	serde_json::to_writer_pretty(writer, value)?;
	Ok(())
}

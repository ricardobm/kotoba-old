//! Importing of dictionary entries from the Yomichan format.

use std::io;
use std::path::Path;

use super::db::tables::Root;

mod yomichan;

/// Import all dictionary files in a given path and append all entries to
/// the [Root].
///
/// This support both zip files and unzipped directories.
///
/// - `import_path` -
///     The root diretory containing either the `.zip` files or unzipped
///     directories for each dictionary.
///
pub fn from_yomichan<P: AsRef<Path>>(db: &mut Root, import_path: P) -> io::Result<()> {
	let import_path = import_path.as_ref();

	let imported = yomichan::import_from(&import_path)?;
	for it in imported {
		it.append_to(db);
	}

	Ok(())
}

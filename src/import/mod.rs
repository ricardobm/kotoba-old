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

	// let mut builder = dict::DictBuilder::new();

	// let mut rng = thread_rng();
	// for mut it in imported {
	// 	{
	// 		it.append_to(&mut builder);
	// 	}

	// 	println!("\n\n{}", it);

	// 	if DUMP_WORD_SAMPLE {
	// 		it.terms.as_mut_slice().shuffle(&mut rng);
	// 		it.kanjis.as_mut_slice().shuffle(&mut rng);
	// 		it.meta_terms.as_mut_slice().shuffle(&mut rng);
	// 		it.meta_kanjis.as_mut_slice().shuffle(&mut rng);

	// 		if it.tags.len() > 0 {
	// 			println!("\n## Tags ##\n");
	// 			for tag in it.tags {
	// 				println!("- {}", tag);
	// 			}
	// 		}

	// 		if it.terms.len() > 0 {
	// 			println!("\n## Terms ##\n");
	// 			for term in it.terms.iter().take(3) {
	// 				println!("\n{}", term);
	// 			}
	// 		}

	// 		if it.kanjis.len() > 0 {
	// 			println!("\n## Kanjis ##\n");
	// 			for kanji in it.kanjis.iter().take(3) {
	// 				println!("\n{}", kanji);
	// 			}
	// 		}

	// 		if it.meta_terms.len() > 0 {
	// 			println!("\n## Meta (Terms) ##\n");
	// 			for meta in it.meta_terms.iter().take(10) {
	// 				println!("- {}", meta);
	// 			}
	// 		}

	// 		if it.meta_kanjis.len() > 0 {
	// 			println!("\n## Meta (Kanjis) ##\n");
	// 			for meta in it.meta_kanjis.iter().take(10) {
	// 				println!("- {}", meta);
	// 			}
	// 		}
	// 	}
	// }

	// let dict = builder.build();

	// println!(
	// 	"\n#\n# Imported {} total entries in {:?}\n#",
	// 	dict.count(),
	// 	start.elapsed()
	// );
	// dict
}

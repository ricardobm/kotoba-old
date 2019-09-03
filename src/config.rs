use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

const DATA_DIR: &str = "hongo-data";

/// Returns the path to the global data directory.
pub fn data_directory() -> &'static Path {
	lazy_static! {
		static ref DATA_PATH: PathBuf = {
			// Find the data directory starting at the current directory
			// and moving up.
			let mut cur_dir = env::current_dir().unwrap();
			let data_path = loop {
				let mut cur_path = PathBuf::from(&cur_dir);
				cur_path.push(DATA_DIR);

				let mut test_path = PathBuf::from(&cur_path);
				test_path.push("import");
				if let Ok(md) = fs::metadata(&test_path) {
					if md.is_dir() {
						break Some(cur_path);
					}
				}

				if let Some(dir) = cur_dir.parent() {
					cur_dir = dir.to_path_buf();
				} else {
					break None;
				}
			};

			match data_path {
				None => {
					eprintln!("\nError: could not find the user directory");
					process::exit(1);
				}
				Some(data_path) => {
					data_path
				}
			}
		};
	}

	DATA_PATH.as_path()
}

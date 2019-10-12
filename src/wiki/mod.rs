use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

#[allow(dead_code)]
pub mod markdown;

use util;

pub struct WikiFile {
	pub name: String,
	pub text: String,
}

pub struct WikiController {
	base_path: PathBuf,
}

impl WikiController {
	pub fn new<P: AsRef<Path>>(base_path: P) -> WikiController {
		WikiController {
			base_path: base_path.as_ref().to_path_buf(),
		}
	}

	/// Retrieve a link from the wiki.
	pub fn get(&self, link: &str) -> util::Result<Option<WikiFile>> {
		if let Some(filename) = Self::file_name(link) {
			let path = self.base_path.join(format!("{}.md", filename));
			if path.exists() {
				Ok(Some(WikiFile {
					name: filename,
					text: fs::read_to_string(path)?,
				}))
			} else {
				Ok(Some(WikiFile {
					name: filename,
					text: String::new(),
				}))
			}
		} else {
			Ok(None)
		}
	}

	/// Save a link to the wiki.
	pub fn save(&mut self, link: &str, content: &str) -> util::Result<Option<WikiFile>> {
		fs::create_dir_all(&self.base_path)?;
		if let Some(filename) = Self::file_name(link) {
			let path = self.base_path.join(format!("{}.md", filename));

			// Write a backup of the current file, if any.
			if path.exists() {
				let backup = self.base_path.join("backups");
				fs::create_dir_all(&backup)?;

				let timestamp = util::DateTime::now();
				let timestamp = timestamp.format("%Y-%m-%d_%H%M%S");
				let backup = backup.join(format!("{}_{}.md", filename, timestamp));
				let current = fs::read_to_string(&path)?;
				if current.len() > 0 {
					fs::write(backup, current)?;
				}
			}

			// Save the file
			fs::write(path, content)?;
			Ok(Some(WikiFile {
				name: filename,
				text: content.to_string(),
			}))
		} else {
			Ok(None)
		}
	}

	/// Returns the file name for a given link.
	pub fn file_name(link: &str) -> Option<String> {
		lazy_static! {
			static ref RE_SPACES: Regex = Regex::new(r"\s+").unwrap();
		}

		let link = link.trim().to_lowercase();
		let link = RE_SPACES.replace_all(&link, "_");
		if !link.chars().all(is_valid_link_char) {
			None
		} else {
			Some(String::from(link))
		}
	}
}

fn is_valid_link_char(chr: char) -> bool {
	chr == '_' || chr.is_alphanumeric()
}

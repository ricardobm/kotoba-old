use std::error::Error;
use std::io;

use data_encoding::HEXLOWER;
use ring::digest::{Context, SHA256};

/// Computes the SHA-256 hash of the input.
pub fn sha256<T: io::Read>(mut input: T) -> io::Result<String> {
	let mut context = Context::new(&SHA256);
	let mut buffer = [0; 1024];

	let digest = loop {
		let size = input.read(&mut buffer)?;
		if size == 0 {
			break context.finish();
		} else {
			context.update(&buffer[..size]);
		}
	};

	let digest = HEXLOWER.encode(digest.as_ref());
	Ok(digest)
}

/// Check the response status, returning an error if it is not successful.
pub fn check_response(response: &reqwest::Response) -> Result<(), Box<dyn Error>> {
	let status = response.status();
	if status.is_success() {
		Ok(())
	} else if let Some(reason) = status.canonical_reason() {
		Err(format!("request failed with status {} ({})", status, reason).into())
	} else {
		Err(format!("request failed with status {}", status).into())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_sha256() {
		assert_eq!(
			"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
			sha256("".as_bytes()).unwrap()
		);
		assert_eq!(
			"b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
			sha256("hello world".as_bytes()).unwrap()
		);
	}
}

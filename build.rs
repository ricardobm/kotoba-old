use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;

extern crate serde;
extern crate serde_json;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Entity {
	pub characters: String,
}

fn main() {
	output().unwrap();
}

fn output() -> std::io::Result<()> {
	let json = include_str!("xtra/entities.json");
	let data: HashMap<String, Entity> = serde_json::from_str(json).unwrap();

	let out_dir = env::var("OUT_DIR").unwrap();
	let dest_path = Path::new(&out_dir).join("markdown_entities.rs");
	let mut f = File::create(&dest_path).unwrap();
	let f = &mut f;

	writeln!(f, r#"lazy_static! {{"#)?;
	writeln!(f, r#"    static ref ENTITIES: HashMap<&'static str, &'static str> = {{"#)?;
	writeln!(f, r#"        let mut out = HashMap::new();"#)?;

	let mut keys = data.keys().collect::<Vec<_>>();
	keys.sort();
	for key in keys {
		writeln!(f, r#"        out.insert({:?}, {:?});"#, key, data[key].characters)?;
	}
	writeln!(f, r#"        out"#)?;
	writeln!(f, r#"    }};"#)?;
	writeln!(f, r#"}}"#)?;
	writeln!(f, r#""#)?;

	println!("{:?}", data);

	Ok(())
}

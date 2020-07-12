#![feature(or_patterns)]

extern crate dict_data;

extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

pub fn main() {
	println!("\nDatabase (version {})\n", dict_data::version());

	let mut rl = Editor::<()>::new();
	loop {
		let input = rl.readline(">> ");
		match input {
			Ok(line) => {
				let line = line.as_str();
				rl.add_history_entry(line);
				println!();
				println!("-> {}", line);
				println!();
			}
			Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
				println!("\nBye!\n");
				break;
			}
			Err(err) => println!("\n   Error: {}\n", err),
		}
	}
}

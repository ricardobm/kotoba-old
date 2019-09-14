#![feature(proc_macro_hygiene, decl_macro)]
#![feature(duration_float)]

#[macro_use]
extern crate serde;
extern crate serde_json;
extern crate serde_tuple;

extern crate bincode;

extern crate fnv;
extern crate itertools;
extern crate rand;
extern crate regex;
extern crate unicode_normalization;

extern crate zip;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

extern crate data_encoding;
extern crate percent_encoding;
extern crate reqwest;
extern crate ring;
extern crate scraper;
extern crate selectors;

extern crate crossbeam;

const DUMP_WORD_SAMPLE: bool = false;

mod app;
mod db;
mod dict;
mod import;
mod japanese;
mod kana;
mod pronunciation;
mod server;
mod util;

use app::App;

fn main() {
	std::process::exit(run());
}

fn run() -> i32 {
	println!();

	let app = App::get();
	if DUMP_WORD_SAMPLE {
		println!();
		app.db().dump(10, &mut std::io::stdout()).unwrap();
		println!();
	}

	server::launch(app);

	0
}

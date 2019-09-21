#![feature(proc_macro_hygiene, decl_macro)]
#![feature(duration_float)]

// Logging

#[macro_use]
extern crate slog;
extern crate slog_scope;
extern crate slog_stdlog;
extern crate slog_term;

// Serde serialization

#[macro_use]
extern crate serde;
extern crate bincode;
extern crate serde_json;
extern crate serde_tuple;

// Utilities

extern crate chrono;
extern crate itertools;
extern crate rand;
extern crate regex;

#[macro_use]
extern crate lazy_static;

// Unicode related

extern crate fnv;
extern crate unicode_normalization;

extern crate zip;

// Server related

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate uuid;

// Web scraping

extern crate data_encoding;
extern crate percent_encoding;
extern crate reqwest;
extern crate ring;
extern crate scraper;
extern crate selectors;

// Concurrent programming

extern crate crossbeam;

// Application modules

#[macro_use]
mod base;

mod api;
mod app;
mod audio;
mod japanese;
mod kana;
mod logging;
mod server;
mod util;
mod wiki;

//
// Main
//

const DUMP_WORD_SAMPLE: bool = false;

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

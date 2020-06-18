// Those are for rocket:
#![feature(proc_macro_hygiene)]
#![feature(decl_macro)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate rocket;
extern crate juniper;
extern crate juniper_rocket;
extern crate rocket_contrib;

#[macro_use]
extern crate rocket_include_static_resources;

mod app;
mod common;
mod graph;
mod graphql;
mod server;

fn main() {
	print!("\nStarting Kotoba server...\n");
	server::launch(app::App::get());
	print!("\nFinished!\n");
}

//! Dictionary search API

use rocket::State;
use rocket_contrib::json::Json;

use app::App;
use japanese;
use japanese::dictionary::{Response, SearchArgs, Source};
use logging::RequestLog;

#[post("/dict/search", data = "<input>")]
pub fn search(log: RequestLog, input: Json<SearchArgs>, app: State<&App>) -> Json<Response> {
	let dict = app.dictionary();
	Json(dict.query(&log, input.into_inner()))
}

#[get("/dict/tags")]
pub fn tags(app: State<&App>) -> Json<Vec<Source>> {
	let dict = app.dictionary();
	Json(dict.tags())
}

#[get("/dict/parse?<q>")]
pub fn parse(app: State<&App>, q: String) -> Json<japanese::ParseResult> {
	let dict = app.dictionary();
	let result = japanese::parse_sentence(dict, &q);
	Json(result)
}

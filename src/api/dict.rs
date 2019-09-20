//! Dictionary search API

use rocket::State;
use rocket_contrib::json::Json;

use app::App;
use logging::RequestLog;
use japanese::dictionary::{Response, SearchArgs, Source};


#[post("/dict/search", data = "<input>")]
pub fn search(log: RequestLog, input: Json<SearchArgs>, app: State<&App>) -> Json<Response> {
	let dict = app.dictionary();
	Json(dict.query(&log, input.into_inner()))
}

#[get("/tags")]
pub fn tags(app: State<&App>) -> Json<Vec<Source>> {
	let dict = app.dictionary();
	Json(dict.tags())
}

use rocket::State;
use rocket_contrib::json::Json;

use app::App;

use japanese;
use logging;
use logging::{RequestLog, ServerLogger};

#[get("/")]
fn index() -> &'static str {
	"Hello world!!!"
}

#[derive(Serialize)]
struct Item {
	pub(self) id:   String,
	pub(self) text: String,
}

#[get("/logs")]
fn logs(app: State<&App>) -> Json<Vec<logging::LogEntry>> {
	Json(app.all_logs())
}

#[get("/log/<req>")]
fn log_by_req(req: logging::RequestId, app: State<&App>) -> Json<Vec<logging::LogEntry>> {
	let cache = app.cache();
	if let Some(entries) = cache.get(&req) {
		let entries: &Vec<logging::LogEntry> = &*entries;
		Json(entries.clone())
	} else {
		Json(vec![])
	}
}

#[get("/list")]
fn list() -> Json<Vec<Item>> {
	let out = vec![
		Item {
			id:   String::from("A"),
			text: String::from("Item A"),
		},
		Item {
			id:   String::from("B"),
			text: String::from("Item B"),
		},
		Item {
			id:   String::from("C"),
			text: String::from("Item C"),
		},
		Item {
			id:   String::from("D"),
			text: String::from("Item D"),
		},
	];
	Json(out)
}

#[post("/search", data = "<input>")]
fn search(log: RequestLog, input: Json<japanese::SearchArgs>, app: State<&App>) -> Json<japanese::QueryResult> {
	let dict = app.dictionary();
	Json(dict.query(&log, &input))
}

#[get("/tags")]
fn tags(dict: State<&japanese::Dictionary>) -> Json<japanese::DbMap> {
	Json(dict.db_map())
}

use api;

pub fn launch(app: &'static App) {
	rocket::ignite()
		.attach(ServerLogger {})
		.manage(app)
		.manage(app.dictionary())
		.mount(
			"/api",
			routes![
				index,
				list,
				search,
				tags,
				logs,
				log_by_req,
				api::audio::query_audio,
				api::audio::get_audio_file,
			],
		)
		.launch();
}

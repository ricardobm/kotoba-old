use rocket::State;
use rocket_contrib::json::Json;

use app::App;

use japanese;
use logging::{RequestLog, ServerLogger};
use pronunciation::JapaneseQuery;
use util;

mod pronunciation;

#[get("/")]
fn index() -> &'static str {
	"Hello world!!!"
}

#[derive(Serialize)]
struct Item {
	pub(self) id:   String,
	pub(self) text: String,
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

pub enum AudioResponse {
	File(String, Vec<u8>),
	NotFound,
}

use rocket::http::hyper::header::{Charset, ContentDisposition, DispositionParam, DispositionType};

impl<'r> rocket::response::Responder<'r> for AudioResponse {
	fn respond_to(self, _request: &rocket::Request) -> rocket::response::Result<'r> {
		use rocket::http::ContentType;
		use rocket::Response;

		let response = match self {
			AudioResponse::File(name, data) => Response::build()
				.header(ContentType::new("audio", "mpeg"))
				.header(ContentDisposition {
					disposition: DispositionType::Inline,
					parameters:  vec![DispositionParam::Filename(
						Charset::Ext("UTF-8".into()),
						None,
						name.into_bytes(),
					)],
				})
				.sized_body(std::io::Cursor::new(data))
				.finalize(),
			AudioResponse::NotFound => Response::build().status(rocket::http::Status::NotFound).finalize(),
		};

		Ok(response)
	}
}

use super::dict;

#[get("/test")]
fn test(log: RequestLog) -> Json<Vec<dict::japanese_pod::Entry>> {
	Json(
		dict::japanese_pod::query_dictionary(
			&log,
			dict::japanese_pod::Args {
				term: "明日".to_string(),
				starts: false,
				..Default::default()
			},
		)
		.unwrap(),
	)
}

#[get("/audio?<kanji>&<kana>")]
fn audio(log: RequestLog, kanji: String, kana: String, app: State<&App>) -> util::Result<AudioResponse> {
	let service = app.pronunciation();
	let result = service.query(
		&log,
		JapaneseQuery {
			term:    kanji.clone(),
			reading: kana.clone(),
			force:   false,
		},
	);

	for err in result.errors {
		error!(log, "{}", err; "kanji" => &kanji, "kana" => &kana);
	}

	if result.items.len() > 0 {
		let first = result.items.into_iter().next().unwrap();
		match first.read() {
			(name, Ok(data)) => Ok(AudioResponse::File(name, data)),
			(_, Err(err)) => Err(err.into()),
		}
	} else {
		Ok(AudioResponse::NotFound)
	}
}

pub fn launch(app: &'static App) {
	rocket::ignite()
		.attach(ServerLogger {})
		.manage(app)
		.manage(app.dictionary())
		.manage(app.pronunciation())
		.mount("/api", routes![index, list, search, tags, audio, test])
		.launch();
}

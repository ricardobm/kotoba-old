use std::error::Error;
use std::sync::Mutex;

use rocket::State;
use rocket_contrib::json::Json;

use japanese;
use pronunciation::{JapaneseQuery, JapaneseService};

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
fn search(input: Json<japanese::SearchArgs>, dict: State<japanese::Dictionary>) -> Json<japanese::QueryResult> {
	Json(dict.query(&input))
}

#[get("/tags")]
fn tags(dict: State<japanese::Dictionary>) -> Json<japanese::DbMap> {
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
fn test() -> Json<Vec<dict::japanese_pod::Entry>> {
	Json(
		dict::japanese_pod::query_dictionary(dict::japanese_pod::Args {
			term: "明日".to_string(),
			starts: false,
			..Default::default()
		})
		.unwrap(),
	)
}

#[get("/audio?<kanji>&<kana>")]
fn audio(kanji: String, kana: String, service: State<Mutex<JapaneseService>>) -> Result<AudioResponse, Box<dyn Error>> {
	let result = service.lock().unwrap().query(JapaneseQuery {
		term:    kanji,
		reading: kana,
		force:   false,
	});

	for it in result.errors {
		eprintln!("Error loading audio: {}", it);
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

pub struct Data {
	pub dict:  japanese::Dictionary,
	pub audio: JapaneseService,
}

pub fn launch(data: Data) {
	rocket::ignite()
		.manage(data.dict)
		.manage(Mutex::new(data.audio))
		.mount("/api", routes![index, list, search, tags, audio, test])
		.launch();
}

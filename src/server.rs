use std::error::Error;

use rocket::State;
use rocket_contrib::json::Json;

use super::dict;
use super::japanese;

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

		let mut response = rocket::Response::new();
		match self {
			AudioResponse::File(name, data) => {
				response.set_header(ContentType::new("audio", "mpeg"));
				response.set_header(ContentDisposition {
					disposition: DispositionType::Inline,
					parameters:  vec![DispositionParam::Filename(
						Charset::Ext("UTF-8".into()),
						None,
						name.into_bytes(),
					)],
				});
				response.set_streamed_body(VecReader(0, data));
			}
			AudioResponse::NotFound => {
				response.set_status(rocket::http::Status::NotFound);
			}
		}

		Ok(response)
	}
}

struct VecReader(usize, Vec<u8>);

impl std::io::Read for VecReader {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		let curr = self.0;
		let data = &self.1;
		let size = std::cmp::min(data.len() - curr, buf.len());
		(&mut buf[..size]).copy_from_slice(&data[curr..curr + size]);
		self.0 += size;
		Ok(size)
	}
}

#[get("/audio?<kanji>&<kana>")]
fn audio(kanji: String, kana: String) -> Result<AudioResponse, Box<dyn Error>> {
	if let Some(audio) = dict::japanese_pod::load_audio(kanji, kana)? {
		Ok(AudioResponse::File("audio.mp3".into(), audio))
	} else {
		Ok(AudioResponse::NotFound)
	}
}

pub fn launch(dict: japanese::Dictionary) {
	rocket::ignite()
		.manage(dict)
		.mount("/api", routes![index, list, search, tags, audio])
		.launch();
}

use rocket::State;

use app::App;
use logging::RequestLog;
use wiki::*;

#[get("/wiki/<link>")]
pub fn get(log: RequestLog, app: State<&App>, link: String) -> WikiResponse {
	let wiki = app.wiki();
	let wiki = wiki.read().unwrap();
	match wiki.get(&link) {
		Ok(None) => WikiResponse::NotFound,
		Ok(Some(file)) => WikiResponse::File(file),
		Err(err) => {
			error!(log, "{}", err);
			WikiResponse::ServerError
		}
	}
}

#[post("/wiki/<link>", data = "<content>")]
pub fn post(log: RequestLog, app: State<&App>, link: String, content: String) -> WikiResponse {
	let wiki = app.wiki();
	let mut wiki = wiki.write().unwrap();
	match wiki.save(&link, &content) {
		Ok(None) => WikiResponse::BadRequest,
		Ok(Some(file)) => WikiResponse::File(file),
		Err(err) => {
			error!(log, "{}", err);
			WikiResponse::ServerError
		}
	}
}

pub enum WikiResponse {
	File(WikiFile),
	NotFound,
	BadRequest,
	ServerError,
}

use rocket::http::hyper::header::{Charset, ContentDisposition, DispositionParam, DispositionType};

impl<'r> rocket::response::Responder<'r> for WikiResponse {
	fn respond_to(self, _request: &rocket::Request) -> rocket::response::Result<'r> {
		use rocket::http::ContentType;
		use rocket::Response;

		let response = match self {
			WikiResponse::File(wiki) => Response::build()
				.header(ContentType::with_params("text", "plain", vec![("charset", "UTF-8")]))
				.header(ContentDisposition {
					disposition: DispositionType::Inline,
					parameters:  vec![DispositionParam::Filename(
						Charset::Ext("UTF-8".into()),
						None,
						format!("{}.md", wiki.name).into_bytes(),
					)],
				})
				.sized_body(std::io::Cursor::new(wiki.text))
				.finalize(),
			WikiResponse::NotFound => Response::build().status(rocket::http::Status::NotFound).finalize(),
			WikiResponse::BadRequest => Response::build().status(rocket::http::Status::BadRequest).finalize(),
			WikiResponse::ServerError => Response::build()
				.status(rocket::http::Status::InternalServerError)
				.finalize(),
		};

		Ok(response)
	}
}

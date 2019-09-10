use rocket::State;
use rocket_contrib::json::Json;

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

pub fn launch(dict: japanese::Dictionary) {
	rocket::ignite()
		.manage(dict)
		.mount("/api", routes![index, list, search])
		.launch();
}

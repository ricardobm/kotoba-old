use rocket::State;
use rocket_contrib::json::Json;

use super::dict::{Dict, SearchMode};

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

#[derive(Serialize)]
struct SearchResult {
	pub count:   usize,
	pub elapsed: f64,
	pub entries: Vec<SearchEntry>,
}

#[derive(Serialize)]
struct SearchEntry {
	pub expression: String,
	pub reading:    String,
	pub forms:      Vec<(String, String)>,
	pub definition: Vec<SearchDefinition>,
}

#[derive(Serialize)]
struct SearchDefinition {
	pub glossary: Vec<String>,
	pub info:     Vec<String>,
	pub tags:     Vec<String>,
}

#[get("/search?<q>")]
fn search(q: String, dict: State<Dict>) -> Json<SearchResult> {
	let start = std::time::Instant::now();
	let results = dict.search(q, SearchMode::Contains);
	let mut entries = Vec::new();
	for it in results {
		let expressions = it.expressions();
		let readings = it.readings();
		let mut result = SearchEntry {
			expression: String::from(expressions[0]),
			reading:    String::from(readings[0]),
			forms:      Vec::new(),
			definition: Vec::new(),
		};
		for (i, expr) in expressions.into_iter().enumerate().skip(1) {
			let expr = String::from(expr);
			let reading = String::from(readings[i]);
			result.forms.push((expr, reading));
		}
		for it in it.definition() {
			let def = SearchDefinition {
				glossary: it.glossary().into_iter().map(String::from).collect(),
				info:     it.info().into_iter().map(String::from).collect(),
				tags:     it.tags().into_iter().map(|x| String::from(x.name())).collect(),
			};
			result.definition.push(def);
		}
		entries.push(result);
	}

	let count = entries.len();
	Json(SearchResult {
		entries: entries,
		count:   count,
		elapsed: start.elapsed().as_secs_f64(),
	})
}

pub fn launch(dict: Dict) {
	rocket::ignite()
		.manage(dict)
		.mount("/api", routes![index, list, search])
		.launch();
}

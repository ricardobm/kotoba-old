use crate::app::App;
use crate::common;
use crate::graph;
use crate::graphql;

use rocket_contrib::json::Json;

#[derive(Serialize)]
struct IndexInfo {
	name: &'static str,
	description: &'static str,
	version: &'static str,
	healthy: bool,
}

#[get("/")]
fn index() -> Json<IndexInfo> {
	let out = IndexInfo {
		name: common::PKG_NAME,
		description: common::PKG_DESCRIPTION,
		version: common::PKG_VERSION,
		healthy: true,
	};
	Json(out)
}

pub fn launch(app: &'static App) {
	rocket::ignite()
		.manage(app)
		.manage(graph::Schema::new(graph::Query, graph::Mutation))
		.mount("/", routes![index])
		.mount("/api", routes![graphql::query, graphql::ide])
		.launch();
}

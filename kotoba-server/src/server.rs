use crate::app::App;
use crate::common;
use crate::graph;
use crate::graphql;

use rocket_contrib::json::Json;
use rocket_include_static_resources::StaticResponse;

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

#[get("/favicon.ico")]
fn favicon() -> StaticResponse {
	static_response!("favicon")
}

pub fn launch(app: &'static App) {
	rocket::ignite()
		.attach(StaticResponse::fairing(|resources| {
			static_resources_initialize!(resources, "favicon", "static/favicon.ico");
		}))
		.manage(app)
		.manage(graph::Schema::new(graph::Query, graph::Mutation))
		.mount("/", routes![index, favicon])
		.mount("/api", routes![graphql::query, graphql::ide])
		.launch();
}

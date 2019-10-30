use app::App;
use logging::RequestLog;

mod dict;
pub use self::dict::DictQuery;

pub struct Context {
	pub app: &'static App,
	pub log: RequestLog,
}

impl juniper::Context for Context {}

/// Root query for the schema.
pub struct Query;

#[juniper::object(Context = Context)]
impl Query {
	/// Application name.
	fn app_name() -> &str {
		"Hongo"
	}

	/// Application version.
	fn app_version() -> &str {
		"0.1"
	}

	/// Query the Japanese dictionary.
	fn dict() -> DictQuery {
		DictQuery
	}
}

pub struct Mutation;

#[juniper::object(Context = Context)]
impl Mutation {
	#[graphql(description = "No-op")]
	fn no_op(context: &Context) -> i32 {
		info!(context.log, "executing no-op");
		42
	}
}

pub type Schema = juniper::RootNode<'static, Query, Mutation>;

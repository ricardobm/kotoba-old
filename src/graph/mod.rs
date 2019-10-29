use app::App;
use logging::RequestLog;

pub struct Context {
	pub app: &'static App,
	pub log: RequestLog,
}

impl juniper::Context for Context {}

pub struct Query;

#[juniper::object(Context = Context)]
impl Query {
	#[graphql(description = "Application name")]
	fn app_name() -> &str {
		"Hongo"
	}

	#[graphql(description = "Application version")]
	fn app_version() -> &str {
		"0.1"
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

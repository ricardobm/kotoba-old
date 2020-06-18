use crate::app::App;
use crate::common;

pub struct Context {
	pub app: &'static App,
}

impl juniper::Context for Context {}

/// Root Query for the GraphQL schema.
pub struct Query;

#[juniper::object(Context = Context)]
impl Query {
	/// Server version.
	fn version() -> &'static str {
		common::PKG_VERSION
	}
}

/// Root Mutation for the GraphQL schema.
pub struct Mutation;

#[juniper::object(Context = Context)]
impl Mutation {
	/// Dummy operation.
	fn no_op() -> i32 {
		42
	}
}

/// Root schema for GraphQL.
pub type Schema = juniper::RootNode<'static, Query, Mutation>;

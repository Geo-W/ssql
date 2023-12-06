pub(crate) mod filter;
mod into_result;
pub(crate) mod query_builder;
pub(crate) mod query_core;
pub(crate) mod stream;
pub(crate) mod ssql_marker;
mod raw_query_builder;

/// Represents different `JOIN` methods in sql.
#[allow(missing_docs)]
pub enum JoinArg {
    Left,
    Right,
    Outer,
    Inner,
}
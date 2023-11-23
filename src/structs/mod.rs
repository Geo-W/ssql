pub(crate) mod filter;
mod into_result;
pub(crate) mod query_builder;
pub(crate) mod query_core;
pub(crate) mod stream;
pub(crate) mod ssql_marker;

pub enum JoinArg {
    Left,
    Right,
    Outer,
    Inner,
}
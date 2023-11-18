pub(crate) mod filter;
pub(crate) mod querybuilder;
pub(crate) mod stream;
pub(crate) mod query_builder_fn;
mod into_result;




pub enum JoinArg{
    Left,
    Right,
    Outer,
    Inner
}
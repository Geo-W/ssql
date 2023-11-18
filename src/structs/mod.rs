use crate::{QueryBuilderI, QueryCore, SsqlMarker};

pub(crate) mod filter;
mod into_result;
pub(crate) mod query_builder_fn;
pub(crate) mod querybuilder;
pub(crate) mod stream;

pub enum JoinArg {
    Left,
    Right,
    Outer,
    Inner,
}

impl<'a, T> QueryBuilderI<'a, T>
where
    T: SsqlMarker,
{
    pub fn new(fields: (&'static str, Vec<&'static str>), func: fn(&str) -> &'static str) -> Self {
        let core = QueryCore::new(fields, func);
        Self {
            core,
            ta: Default::default(),
        }
    }
}

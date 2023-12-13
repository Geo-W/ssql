use polars::frame::DataFrame;
use crate::structs::ssql_marker::SsqlMarker;
use serde_json::Value;
use tiberius::{QueryStream, Row};
use crate::SsqlResult;

pub trait IntoResult
where
    Self::Js: Send + Sync,
{
    type Js;
    type Df;
    fn to_struct(r: &Row) -> Self
    where
        Self: Sized + 'static;

    fn to_json(r: &Row) -> Self::Js
    where
        Self: Sized;

    fn df(v: QueryStream) -> SsqlResult<Self::Df> where Self: Sized;
}

impl<Ta> IntoResult for Ta
where
    Ta: SsqlMarker,
{
    type Js = Value;

    type Df = DataFrame;
    fn to_struct(r: &Row) -> Self
    where
        Self: Sized + 'static,
    {
        Ta::row_to_struct(r)
    }

    fn to_json(r: &Row) -> Value
    where
        Self: Sized,
    {
        Ta::row_to_json(r).into()
    }

    fn df(v: QueryStream) -> SsqlResult<Self::Df> where Self: Sized {
        Ok(futures_lite::future::block_on(Ta::dataframe(v))?)
    }
}

impl<Ta, Tb> IntoResult for (Ta, Tb)
where
    Ta: SsqlMarker,
    Tb: SsqlMarker,
{
    type Js = (Value, Value);
    type Df = ();

    fn to_struct(r: &Row) -> Self
    where
        Self: Sized + 'static,
    {
        (Ta::row_to_struct(r), Tb::row_to_struct(r))
    }

    fn to_json(r: &Row) -> Self::Js
    where
        Self: Sized,
    {
        (Ta::row_to_json(r).into(), Tb::row_to_json(r).into())
    }

    fn df(v: QueryStream) -> SsqlResult<Self::Df> where Self: Sized {
        todo!()
    }
}

impl<Ta, Tb, Tc> IntoResult for (Ta, Tb, Tc)
where
    Ta: SsqlMarker,
    Tb: SsqlMarker,
    Tc: SsqlMarker,
{
    type Js = (Value, Value, Value);
    type Df = ();

    fn to_struct(r: &Row) -> Self
    where
        Self: Sized + 'static,
    {
        (
            Ta::row_to_struct(r),
            Tb::row_to_struct(r),
            Tc::row_to_struct(r),
        )
    }

    fn to_json(r: &Row) -> Self::Js
    where
        Self: Sized,
    {
        (
            Ta::row_to_json(r).into(),
            Tb::row_to_json(r).into(),
            Tc::row_to_json(r).into(),
        )
    }

    fn df(v: QueryStream) -> SsqlResult<Self::Df> where Self: Sized {
        todo!()
    }
}

impl<Ta, Tb, Tc, Td> IntoResult for (Ta, Tb, Tc,Td)
where
    Ta: SsqlMarker,
    Tb: SsqlMarker,
    Tc: SsqlMarker,
    Td: SsqlMarker,
{
    type Js = (Value, Value, Value, Value);
    type Df = ();

    fn to_struct(r: &Row) -> Self
    where
        Self: Sized + 'static,
    {
        (
            Ta::row_to_struct(r),
            Tb::row_to_struct(r),
            Tc::row_to_struct(r),
            Td::row_to_struct(r),
        )
    }

    fn to_json(r: &Row) -> Self::Js
    where
        Self: Sized,
    {
        (
            Ta::row_to_json(r).into(),
            Tb::row_to_json(r).into(),
            Tc::row_to_json(r).into(),
            Td::row_to_json(r).into(),
        )
    }

    fn df(v: QueryStream) -> SsqlResult<Self::Df> where Self: Sized {
        todo!()
    }
}

impl<Ta, Tb, Tc, Td, Te> IntoResult for (Ta, Tb, Tc,Td, Te)
where
    Ta: SsqlMarker,
    Tb: SsqlMarker,
    Tc: SsqlMarker,
    Td: SsqlMarker,
    Te: SsqlMarker
{
    type Js = (Value, Value, Value, Value, Value);
    type Df = ();

    fn to_struct(r: &Row) -> Self
    where
        Self: Sized + 'static,
    {
        (
            Ta::row_to_struct(r),
            Tb::row_to_struct(r),
            Tc::row_to_struct(r),
            Td::row_to_struct(r),
            Te::row_to_struct(r),
        )
    }

    fn to_json(r: &Row) -> Self::Js
    where
        Self: Sized,
    {
        (
            Ta::row_to_json(r).into(),
            Tb::row_to_json(r).into(),
            Tc::row_to_json(r).into(),
            Td::row_to_json(r).into(),
            Te::row_to_json(r).into(),
        )
    }

    fn df(v: QueryStream) -> SsqlResult<Self::Df> where Self: Sized {
        todo!()
    }
}

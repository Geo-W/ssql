#[cfg(feature = "polars")]
use polars::frame::DataFrame;
#[cfg(feature = "serde")]
use serde_json::Value;
#[cfg(feature = "polars")]
use tiberius::QueryStream;
use tiberius::Row;

#[cfg(feature = "polars")]
use crate::SsqlResult;
use crate::structs::ssql_marker::SsqlMarker;

pub trait IntoResult
{
    #[cfg(feature = "serde")]
    type Js: Send + Sync;

    fn to_struct(r: &Row) -> Self
    where
        Self: Sized + 'static;

    #[cfg(feature = "serde")]
    fn to_json(r: &Row) -> Self::Js
    where
        Self: Sized;

    #[cfg(feature = "polars")]
    type Df;

    #[cfg(feature = "polars")]
    fn df(v: QueryStream) -> SsqlResult<Self::Df>
    where
        Self: Sized;
}

impl<Ta> IntoResult for Ta
where
    Ta: SsqlMarker,
{
    #[cfg(feature = "serde")]
    type Js = Value;

    #[cfg(feature = "polars")]
    type Df = DataFrame;
    fn to_struct(r: &Row) -> Self
    where
        Self: Sized + 'static,
    {
        Ta::row_to_struct(r)
    }

    #[cfg(feature = "serde")]
    fn to_json(r: &Row) -> Value
    where
        Self: Sized,
    {
        Ta::row_to_json(r).into()
    }

    #[cfg(feature = "polars")]
    fn df(v: QueryStream) -> SsqlResult<Self::Df>
    where
        Self: Sized,
    {
        Ok(futures_lite::future::block_on(Ta::dataframe(v))?)
    }
}

impl<Ta, Tb> IntoResult for (Ta, Tb)
where
    Ta: SsqlMarker,
    Tb: SsqlMarker,
{
    #[cfg(feature = "serde")]
    type Js = (Value, Value);

    fn to_struct(r: &Row) -> Self
    where
        Self: Sized + 'static,
    {
        (Ta::row_to_struct(r), Tb::row_to_struct(r))
    }

    #[cfg(feature = "serde")]
    fn to_json(r: &Row) -> Self::Js
    where
        Self: Sized,
    {
        (Ta::row_to_json(r).into(), Tb::row_to_json(r).into())
    }

    #[cfg(feature = "polars")]
    type Df = ();

    #[cfg(feature = "polars")]
    fn df(v: QueryStream) -> SsqlResult<Self::Df>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl<Ta, Tb, Tc> IntoResult for (Ta, Tb, Tc)
where
    Ta: SsqlMarker,
    Tb: SsqlMarker,
    Tc: SsqlMarker,
{
    #[cfg(feature = "serde")]
    type Js = (Value, Value, Value);

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

    #[cfg(feature = "serde")]
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

    #[cfg(feature = "polars")]
    type Df = ();

    #[cfg(feature = "polars")]
    fn df(v: QueryStream) -> SsqlResult<Self::Df>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl<Ta, Tb, Tc, Td> IntoResult for (Ta, Tb, Tc, Td)
where
    Ta: SsqlMarker,
    Tb: SsqlMarker,
    Tc: SsqlMarker,
    Td: SsqlMarker,
{
    #[cfg(feature = "serde")]
    type Js = (Value, Value, Value, Value);

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

    #[cfg(feature = "serde")]
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

    #[cfg(feature = "polars")]
    type Df = ();

    #[cfg(feature = "polars")]
    fn df(v: QueryStream) -> SsqlResult<Self::Df>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl<Ta, Tb, Tc, Td, Te> IntoResult for (Ta, Tb, Tc, Td, Te)
where
    Ta: SsqlMarker,
    Tb: SsqlMarker,
    Tc: SsqlMarker,
    Td: SsqlMarker,
    Te: SsqlMarker,
{
    #[cfg(feature = "serde")]
    type Js = (Value, Value, Value, Value, Value);

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

    #[cfg(feature = "serde")]
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

    #[cfg(feature = "polars")]
    type Df = ();

    #[cfg(feature = "polars")]
    fn df(v: QueryStream) -> SsqlResult<Self::Df>
    where
        Self: Sized,
    {
        todo!()
    }
}

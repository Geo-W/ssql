#[cfg(feature = "polars")]
use polars::frame::DataFrame;
#[cfg(feature = "serde")]
use serde_json::Value;
#[cfg(feature = "polars")]
use tiberius::QueryStream;
use tiberius::Row;

use crate::structs::ssql_marker::SsqlMarker;
#[cfg(feature = "polars")]
use crate::SsqlResult;

pub trait IntoResult {
    fn to_struct(r: &Row) -> Self
    where
        Self: Sized + 'static;

    #[cfg(feature = "serde")]
    type Js: Send + Sync;

    #[cfg(feature = "serde")]
    fn to_json(r: &Row) -> Self::Js
    where
        Self: Sized;

    #[cfg(feature = "polars")]
    type Df;

    #[cfg(feature = "polars")]
    fn df(v: QueryStream<'_>) -> impl std::future::Future<Output = SsqlResult<Self::Df>> + Send
    where
        Self: Sized;
}

impl<Ta> IntoResult for Ta
where
    Ta: SsqlMarker,
{
    fn to_struct(r: &Row) -> Self
    where
        Self: Sized + 'static,
    {
        Ta::row_to_struct(r)
    }

    #[cfg(feature = "serde")]
    type Js = Value;
    #[cfg(feature = "serde")]
    fn to_json(r: &Row) -> Value
    where
        Self: Sized,
    {
        Ta::row_to_json(r).into()
    }

    #[cfg(feature = "polars")]
    type Df = DataFrame;

    #[cfg(feature = "polars")]
    async fn df(v: QueryStream<'_>) -> SsqlResult<Self::Df>
    where
        Self: Sized,
    {
        Ta::dataframe(v).await
    }
}

impl<Ta, Tb> IntoResult for (Ta, Tb)
where
    Ta: SsqlMarker,
    Tb: SsqlMarker,
{
    fn to_struct(r: &Row) -> Self
    where
        Self: Sized + 'static,
    {
        (Ta::row_to_struct(r), Tb::row_to_struct(r))
    }

    #[cfg(feature = "serde")]
    type Js = (Value, Value);

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
    async fn df(v: QueryStream<'_>) -> SsqlResult<Self::Df>
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
    type Js = (Value, Value, Value);

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
    async fn df(v: QueryStream<'_>) -> SsqlResult<()>
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
    type Js = (Value, Value, Value, Value);

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
    async fn df(v: QueryStream<'_>) -> SsqlResult<()>
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
    type Js = (Value, Value, Value, Value, Value);

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
    async fn df(v: QueryStream<'_>) -> SsqlResult<()>
    where
        Self: Sized,
    {
        todo!()
    }
}

use std::marker::PhantomData;

use async_trait::async_trait;
use futures_lite::StreamExt;
use tokio::net::TcpStream;
use tokio_util::compat::Compat;

use crate::structs::into_result::IntoResult;
use crate::structs::query_core::{Executable, QueryCore};
use crate::structs::ssql_marker::SsqlMarker;
use crate::structs::JoinArg;
use crate::{ColExpr, FilterExpr, RowStream, SsqlResult};

pub trait CoreVisitor<'a> {
    fn core_mut(&mut self) -> &mut QueryCore<'a>;
    fn core_ref(&self) -> &QueryCore<'a>;
}

/// Core trait for constructing and conducting query.
#[async_trait]
pub trait QueryAble<'a>: Send + Sync + CoreVisitor<'a>
where
    Self::Ret: IntoResult + Send + Sync + 'static,
{
    #[doc(hidden)]
    type NxtModel<NxtType: SsqlMarker>;
    #[doc(hidden)]
    type Ret;

    /// Getting data from query builder instance, will panic if data type defined in struct is not corresponding to the tables.
    /// Returns Vector containing tuple of TABLE structs `Vec<(Ta..Te)>`, depends on how much tables joined in this query builder.
    async fn all(
        &self,
        conn: &mut tiberius::Client<Compat<TcpStream>>,
    ) -> SsqlResult<Vec<Self::Ret>> {
        let mut stream = self.core_ref().execute(conn).await?.into_row_stream();
        let mut ret = vec![];
        while let Some(row) = stream.try_next().await? {
            ret.push(Self::Ret::to_struct(&row));
        }
        Ok(ret)
    }

    /// Similar to [`all`], but returns a stream producing tuple of structs instead of a whole vector.
    ///
    /// [`all`]: trait.QueryAble.html#method.all
    async fn stream<'b>(
        &self,
        conn: &'b mut tiberius::Client<Compat<TcpStream>>,
    ) -> SsqlResult<RowStream<'b, Self::Ret>> {
        let stream = self.core_ref().execute(conn).await?;
        Ok(RowStream::new(stream, Self::Ret::to_struct))
    }

    /// Similar to [`all`], but returns first row only.
    ///
    /// [`all`]: trait.QueryAble.html#method.all
    async fn one(
        &self,
        conn: &mut tiberius::Client<Compat<TcpStream>>,
    ) -> SsqlResult<Option<Self::Ret>> {
        let row = self.core_ref().execute(conn).await?.into_row().await?;
        match row {
            None => Ok(None),
            Some(row) => Ok(Some(Self::Ret::to_struct(&row))),
        }
    }

    /// Similar to [`all`], but returns Vector containing tuple of [`Value`] instead of struct itself.
    ///
    /// [`all`]: trait.QueryAble.html#method.all
    /// [`Value`]: serde_json::Value
    #[cfg(feature = "serde")]
    async fn json(
        &self,
        conn: &mut tiberius::Client<Compat<TcpStream>>,
    ) -> SsqlResult<Vec<<<Self as QueryAble<'a>>::Ret as IntoResult>::Js>> {
        let mut stream = self.core_ref().execute(conn).await?.into_row_stream();
        let mut ret = vec![];
        while let Some(row) = stream.try_next().await? {
            ret.push(Self::Ret::to_json(&row))
        }
        Ok(ret)
    }

    /// Similar to [`all`], but returns [`Polars DataFrame`] representing the query result.
    ///
    /// [`all`]: trait.QueryAble.html#method.all
    /// [`Polars DataFrame`]: polars::prelude::DataFrame
    #[cfg(feature = "polars")]
    async fn df(
        &self,
        conn: &mut tiberius::Client<Compat<TcpStream>>,
    ) -> SsqlResult<<Self::Ret as IntoResult>::Df> {
        // let all = self.all(conn).await?;
        Self::Ret::df(self.core_ref().execute(conn).await?)
    }

    /// Perform left join on another table.
    /// Will panic if the relationship not presented in field attribute `#[ssql(foreign_key=...)]`
    /// or if the provided table is already joined.
    /// ```no_run
    /// # use ssql::prelude::*;
    /// #[derive(ORM)]
    /// #[ssql(table = person, schema = SCHEMA1)]
    /// struct Person {
    ///     #[ssql(primary_key)]
    ///     id: i32,
    ///     email: Option<String>,
    /// }
    ///
    /// #[derive(ORM)]
    /// #[ssql(table = posts)]
    /// struct Posts {
    ///     id: i32,
    ///     post: String,
    ///     #[ssql(foreign_key = "SCHEMA1.Person.id")]
    ///     person_id: i32,
    /// }
    /// let _ = Person::query().join::<Posts>(ssql::JoinArg::Left);
    /// let _ = Person::query().left_join::<Posts>(); //same as above
    /// //SQL: `... FROM SCHEMA1.person LEFT JOIN posts ON SCHEMA1.Person.id = posts.person_id`
    /// ```
    fn join<NxtType>(self, join_args: JoinArg) -> Self::NxtModel<NxtType>
    where
        NxtType: SsqlMarker;

    /// See [`join`]. Except that this method only perform `LEFT JOIN`.
    ///
    /// [`join`]: trait.QueryAble.html#tymethod.join
    fn left_join<NxtType>(self) -> Self::NxtModel<NxtType>
    where
        NxtType: SsqlMarker,
        Self: Sized,
    {
        self.join::<NxtType>(JoinArg::Left)
    }

    /// See [`join`]. Except that this method only perform `RIGHT JOIN`.
    ///
    /// [`join`]: trait.QueryAble.html#tymethod.join
    fn right_join<NxtType>(self) -> Self::NxtModel<NxtType>
    where
        NxtType: SsqlMarker,
        Self: Sized,
    {
        self.join::<NxtType>(JoinArg::Right)
    }

    /// See [`join`]. Except that this method only perform `INNER JOIN`.
    ///
    /// [`join`]: trait.QueryAble.html#tymethod.join
    fn inner_join<NxtType>(self) -> Self::NxtModel<NxtType>
    where
        NxtType: SsqlMarker,
        Self: Sized,
    {
        self.join::<NxtType>(JoinArg::Inner)
    }

    /// See [`join`]. Except that this method only perform `OUTER JOIN`.
    ///
    /// [`join`]: trait.QueryAble.html#tymethod.join
    fn outer_join<NxtType>(self) -> Self::NxtModel<NxtType>
    where
        NxtType: SsqlMarker,
        Self: Sized,
    {
        self.join::<NxtType>(JoinArg::Outer)
    }

    /// Chain a filter to current builder.
    /// This method will check whether the table provided is in this builder thus [`SsqlResult`] is returned.
    fn filter(mut self, filter_expr: FilterExpr<'a>) -> SsqlResult<Self>
    where
        Self: Sized,
    {
        self.core_mut().filter(filter_expr)?;
        Ok(self)
    }

    /// Ordering the output by a specified column in ascending order.
    fn order_by_asc(mut self, col_expr: ColExpr) -> SsqlResult<Self>
    where
        Self: Sized,
    {
        self.core_mut().order_by(col_expr, true)?;
        Ok(self)
    }

    /// Ordering the output by a specified column in descending order.
    fn order_by_desc(mut self, col_expr: ColExpr) -> SsqlResult<Self>
    where
        Self: Sized,
    {
        self.core_mut().order_by(col_expr, false)?;
        Ok(self)
    }
}

/// Struct representing one `TABLE`.
pub struct QueryBuilderI<'a, Ta>
where
    Ta: SsqlMarker,
{
    core: QueryCore<'a>,
    ta: PhantomData<Ta>,
}

impl<'a, T> QueryBuilderI<'a, T>
where
    T: SsqlMarker,
{
    /// Create a new query builder, shouldn't call it manually, this is handled by [`query`] method.
    ///
    /// ['query`]: trait.SsqlMarker.html#tymethod.query
    pub fn new(fields: (&'static str, Vec<&'static str>), func: fn(&str) -> &'static str) -> Self {
        let core = QueryCore::new(fields, func);
        Self {
            core,
            ta: Default::default(),
        }
    }
}

pub struct QueryBuilderII<'a, Ta, Tb>
where
    Ta: SsqlMarker,
    Tb: SsqlMarker,
{
    core: QueryCore<'a>,
    ta: PhantomData<Ta>,
    tb: PhantomData<Tb>,
}

pub struct QueryBuilderIII<'a, Ta, Tb, Tc>
where
    Ta: SsqlMarker,
    Tb: SsqlMarker,
    Tc: SsqlMarker,
{
    core: QueryCore<'a>,
    ta: PhantomData<Ta>,
    tb: PhantomData<Tb>,
    tc: PhantomData<Tc>,
}

pub struct QueryBuilderIV<'a, Ta, Tb, Tc, Td>
where
    Ta: SsqlMarker,
    Tb: SsqlMarker,
    Tc: SsqlMarker,
    Td: SsqlMarker,
{
    core: QueryCore<'a>,
    ta: PhantomData<Ta>,
    tb: PhantomData<Tb>,
    tc: PhantomData<Tc>,
    td: PhantomData<Td>,
}

pub struct QueryBuilderV<'a, Ta, Tb, Tc, Td, Te>
where
    Ta: SsqlMarker,
    Tb: SsqlMarker,
    Tc: SsqlMarker,
    Td: SsqlMarker,
    Te: SsqlMarker,
{
    core: QueryCore<'a>,
    ta: PhantomData<Ta>,
    tb: PhantomData<Tb>,
    tc: PhantomData<Tc>,
    td: PhantomData<Td>,
    te: PhantomData<Te>,
}

#[async_trait]
impl<'a, Ta> QueryAble<'a> for QueryBuilderI<'a, Ta>
where
    Ta: SsqlMarker + Send + Sync + 'static,
    // QueryCore<'a, Ta, NormalQuery>: Send + Executable,
{
    type NxtModel<NxtType: SsqlMarker> = QueryBuilderII<'a, Ta, NxtType>;

    type Ret = Ta;

    fn join<NxtType>(self, join_args: JoinArg) -> Self::NxtModel<NxtType>
    where
        NxtType: SsqlMarker,
    {
        QueryBuilderII {
            core: self.core.join::<NxtType>(join_args),
            ta: Default::default(),
            tb: Default::default(),
        }
    }
}

impl_queryable!(QueryBuilderII, QueryBuilderIII, [Ta, Tb], [ta, tb, tc]);
impl_queryable!(QueryBuilderIII, QueryBuilderIV, [Ta, Tb, Tc], [ta, tb, tc, td]);
impl_queryable!(QueryBuilderIV, QueryBuilderV, [Ta, Tb, Tc, Td], [ta, tb, tc, td, te]);

#[async_trait]
impl<'a, Ta, Tb, Tc, Td, Te> QueryAble<'a> for QueryBuilderV<'a, Ta, Tb,Tc,Td,Te>
where
    Ta: SsqlMarker + Send + Sync + 'static,
    Tb: SsqlMarker + Send + Sync + 'static,
    Tc: SsqlMarker + Send + Sync + 'static,
    Td: SsqlMarker + Send + Sync + 'static,
    Te: SsqlMarker + Send + Sync + 'static,
{
    type NxtModel<NxtType: SsqlMarker> = PhantomData<NxtType>;

    type Ret = (Ta, Tb, Tc, Td, Te);

    fn join<NxtType>(self, _join_args: JoinArg) -> Self::NxtModel<NxtType>
    where
        NxtType: SsqlMarker,
    {
        unimplemented!()
    }
}

impl<'a, Ta> CoreVisitor<'a> for QueryBuilderI<'a, Ta>
where
    Ta: SsqlMarker,
{
    fn core_mut(&mut self) -> &mut QueryCore<'a> {
        &mut self.core
    }

    fn core_ref(&self) -> &QueryCore<'a> {
        &self.core
    }
}
impl_corevisitor!(QueryBuilderII, [Ta, Tb]);
impl_corevisitor!(QueryBuilderIII, [Ta, Tb, Tc]);
impl_corevisitor!(QueryBuilderIV, [Ta, Tb, Tc, Td]);
impl_corevisitor!(QueryBuilderV, [Ta, Tb, Tc, Td, Te]);

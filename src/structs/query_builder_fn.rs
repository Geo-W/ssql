use std::marker::PhantomData;

use async_trait::async_trait;
use futures_lite::StreamExt;
use tokio::net::TcpStream;
use tokio_util::compat::Compat;

use crate::{ColExpr, FilterExpr, QueryCore, RowStream, SsqlMarker, SsqlResult};
use crate::structs::into_result::IntoResult;
use crate::structs::JoinArg;
use crate::structs::querybuilder::Executable;

pub trait CoreVisitor<'a> {
    fn core_mut(&mut self) -> &mut QueryCore<'a>;
    fn core_ref(&self) -> &QueryCore<'a>;
}

#[async_trait]
pub trait QueryAble<'a>: Send + Sync + CoreVisitor<'a>
where
    Self::Ret: IntoResult + Send + Sync + 'static,
{
    #[doc(hidden)]
    type NxtModel<NxtType: SsqlMarker>;
    #[doc(hidden)]
    type Ret;

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

    async fn stream<'b>(
        &self,
        conn: &'b mut tiberius::Client<Compat<TcpStream>>,
    ) -> SsqlResult<RowStream<'b, Self::Ret>> {
        let stream = self.core_ref().execute(conn).await?;
        Ok(RowStream::new(stream, Self::Ret::to_struct))
    }

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
    /// let _ = Person::query().left_join::<Posts>();
    /// ```
    /// SQL: `... FROM SCHEMA1.person LEFT JOIN posts ON SCHEMA1.Person.id = posts.person_id`
    fn join<NxtType>(self, join_args: JoinArg) -> Self::NxtModel<NxtType>
    where
        NxtType: SsqlMarker;

    fn left_join<NxtType>(self) -> Self::NxtModel<NxtType>
    where
        NxtType: SsqlMarker,
        Self: Sized,
    {
        self.join::<NxtType>(JoinArg::Left)
    }

    fn right_join<NxtType>(self) -> Self::NxtModel<NxtType>
    where
        NxtType: SsqlMarker,
        Self: Sized,
    {
        self.join::<NxtType>(JoinArg::Right)
    }

    fn inner_join<NxtType>(self) -> Self::NxtModel<NxtType>
    where
        NxtType: SsqlMarker,
        Self: Sized,
    {
        self.join::<NxtType>(JoinArg::Inner)
    }

    fn outer_join<NxtType>(self) -> Self::NxtModel<NxtType>
    where
        NxtType: SsqlMarker,
        Self: Sized,
    {
        self.join::<NxtType>(JoinArg::Outer)
    }

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

pub struct QueryBuilderI<'a, Ta>
where
    Ta: SsqlMarker,
{
    pub core: QueryCore<'a>,
    pub ta: PhantomData<Ta>,
}

pub struct QueryBuilderII<'a, Ta, Tb>
where
    Ta: SsqlMarker,
    Tb: SsqlMarker,
{
    pub core: QueryCore<'a>,
    ta: PhantomData<Ta>,
    pub tb: PhantomData<Tb>,
}

pub struct QueryBuilderIII<'a, Ta, Tb, Tc>
where
    Ta: SsqlMarker,
    Tb: SsqlMarker,
    Tc: SsqlMarker,
{
    core: QueryCore<'a, Ta>,
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

#[async_trait]
impl<'a, Ta, Tb> QueryAble<'a> for QueryBuilderII<'a, Ta, Tb>
where
    Ta: SsqlMarker + Send + Sync + 'static,
    Tb: SsqlMarker + Send + Sync + 'static,
{
    type NxtModel<NxtType: SsqlMarker> = QueryBuilderII<'a, Ta, NxtType>;

    type Ret = (Ta, Tb);

    fn join<NxtType>(self, join_args: JoinArg) -> Self::NxtModel<NxtType>
    where
        NxtType: SsqlMarker,
    {
        todo!()
        // QueryBuilderIII {
        //     core: self.core.left_join::<NxtType>(),
        //     ta: Default::default(),
        //     tb: Default::default(),
        //     tc: Default::default(),
        // }
    }
}

// #[async_trait]
// impl<'a, FN, Ret, Ta, Tb> QueryAble<'a> for QueryBuilderII<'a, FN, Ret, Ta, Tb>
// where
//     Ta: SsqlMarker + Send + Sync,
//     Tb: SsqlMarker + Send + Sync,
//     FN: Fn(&Row) -> Ret + 'static + Send + Sync,
//     // QueryCore<'a, Ta, NormalQuery>: Send + Executable,
//     Ret: Send + Sync,
// {
//     type NxtModel<NxtType: SsqlMarker> = QueryBuilderIII<'a, FN, Ret, Ta, Tb, NxtType>;
//     type NewFnModel<NewFN, NewRet> = QueryBuilderII<'a, NewFN, NewRet, Ta, Tb>
//     where
//         NewFN: Fn(&Row) -> NewRet + 'static + Send + Sync;
//     type Ret = SsqlResult<Vec<(Ret, Ret)>>;
//
//     fn join<NxtType>(self) -> Self::NxtModel<NxtType>
//     where
//         NxtType: SsqlMarker,
//     {
//         QueryBuilderIII {
//             a: self.a.left_join::<NxtType>(),
//             tb: Default::default(),
//             tc: Default::default(),
//             func: self.func,
//         }
//     }
//
//     async fn all(&self, conn: &mut Client<Compat<TcpStream>>) -> Self::Ret {
//         self.a
//             .exec(conn, |x| ((&self.func)(x), (&self.func)(x)))
//             .await
//     }
//
//     fn filter(mut self, filter_expr: FilterExpr<'a>) -> SsqlResult<Self>
//     where
//         Self: Sized,
//     {
//         self.a = self.a.filter(filter_expr)?;
//         Ok(self)
//     }
//
//     fn replace_fn<NewFN, NewRet>(self, new_fn: NewFN) -> Self::NewFnModel<NewFN, NewRet>
//     where
//         NewFN: Fn(&Row) -> NewRet + 'static + Send + Sync,
//     {
//         Self::NewFnModel {
//             a: self.a,
//             tb: self.tb,
//             func: new_fn,
//         }
//     }
// }
//
// impl_queryable!(
//     QueryBuilderIII,
//     QueryBuilderIV,
//     [Ta, Tb, Tc],
//     [Ret, Ret, Ret],
//     [td, tc, tb],
//     [func, func, func]
// );

//
// impl<'a, FN, Ret, Ta> QueryBuilderI<'a, FN, Ret, Ta>
// where
//     Ta: SsqlMarker,
//     FN: Fn(&Row) -> Ret + 'static + Send + Sync,
// {
//     pub fn new(s: QueryCore<'a, Ta>, func: FN) -> Self {
//         Self { a: s, func }
//     }
//
//     pub fn replace_fn<NewRet, NewFN>(self, new_fn: NewFN) -> QueryBuilderI<'a, NewFN, NewRet, Ta>
//     where
//         NewFN: Fn(&Row) -> NewRet + 'static + Send + Sync,
//     {
//         QueryBuilderI {
//             a: self.a,
//             func: new_fn,
//         }
//     }
// }
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

impl<'a, Ta, Tb> CoreVisitor<'a> for QueryBuilderII<'a, Ta, Tb>
where
    Ta: SsqlMarker,
    Tb: SsqlMarker,
{
    fn core_mut(&mut self) -> &mut QueryCore<'a> {
        &mut self.core
    }

    fn core_ref(&self) -> &QueryCore<'a> {
        &self.core
    }
}

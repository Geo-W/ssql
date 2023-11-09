use crate::structs::querybuilder::{Executable, NormalQuery};
use crate::structs::JoinArg;
use crate::{ColExpr, FilterExpr, QueryCore, SsqlMarker, SsqlResult};
use async_trait::async_trait;
use futures_lite::StreamExt;
use serde_json::{Map, Value};
use std::marker::PhantomData;
use tiberius::{Client, Query, QueryItem, Row};
use tokio::net::TcpStream;
use tokio_util::compat::Compat;

pub trait CoreVisitor<'a> {
    fn core(&mut self) -> &mut QueryCore<'a>;
    // fn func<Ret>(&self) -> &dyn Fn(&Row) -> Ret + Send + Sync;
}

#[async_trait]
pub trait QueryAble<'a>: Send + Sync + CoreVisitor<'a> {
    #[doc(hidden)]
    type NxtModel<NxtType: SsqlMarker>;
    #[doc(hidden)]
    type Ret;
    #[doc(hidden)]
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
    async fn all(&self, conn: &mut tiberius::Client<Compat<TcpStream>>) -> SsqlResult<Vec<Self::Ret>>;

    fn filter(mut self, filter_expr: FilterExpr<'a>) -> SsqlResult<Self>
    where
        Self: Sized,
    {
        self.core().filter(filter_expr)?;
        Ok(self)
    }

    fn order_by_asc(mut self, col_expr: ColExpr) -> SsqlResult<Self>
    where
        Self: Sized,
    {
        self.core().order_by_asc(col_expr)?;
        Ok(self)
    }

    fn order_by_desc(mut self, col_expr: ColExpr) -> SsqlResult<Self>
    where
        Self: Sized,
    {
        self.core().order_by_desc(col_expr)?;
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

impl<'a> QueryCore<'a, NormalQuery>
where
    // Ta: SsqlMarker + Send + Sync,
    QueryCore<'a, NormalQuery>: Send + Executable,
{
    async fn exec<F, Ret>(
        &self,
        conn: &mut Client<Compat<TcpStream>>,
        func: F,
    ) -> SsqlResult<(Vec<Ret>)>
    where
        F: Fn(&tiberius::Row) -> Ret + Send + Sync,
        Ret: Send + Sync,
    {
        let mut stream = self.execute(conn).await?;

        let mut vec = vec![];

        while let Some(item) = stream.try_next().await.unwrap() {
            match item {
                QueryItem::Row(row) => vec.push(func(&row)),
                QueryItem::Metadata(_) => {}
            }
        }

        Ok(vec)
    }
}

#[async_trait]
impl<'a, Ta> QueryAble<'a> for QueryBuilderI<'a, Ta>
where
    Ta: SsqlMarker + Send + Sync,
    // QueryCore<'a, Ta, NormalQuery>: Send + Executable,
{
    type NxtModel<NxtType: SsqlMarker> = QueryBuilderII<'a, Ta, NxtType>;

    type Ret = Ta;

    fn join<NxtType>(self, join_args: JoinArg) -> Self::NxtModel<NxtType>
    where
        NxtType: SsqlMarker,
    {
        QueryBuilderII {
            core: self.core.left_join::<NxtType>(),
            ta: Default::default(),
            tb: Default::default(),
        }
    }

    async fn all(&self, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<Vec<Self::Ret>> {
        self.core.exec(conn, Ta::row_to_struct).await
    }
}

#[async_trait]
impl<'a, Ta, Tb> QueryAble<'a> for QueryBuilderII<'a, Ta, Tb>
where
    Ta: SsqlMarker + Send + Sync,
    Tb: SsqlMarker + Send + Sync,
    // QueryCore<'a, Ta, NormalQuery>: Send + Executable,
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

    async fn all(&self, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<Vec<Self::Ret>> {
        self.core
            .exec(conn, |x| (Ta::row_to_struct(x), Tb::row_to_struct(x)))
            .await
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
    fn core(&mut self) -> &mut QueryCore<'a> {
        &mut self.core
    }
}

impl<'a, Ta, Tb> CoreVisitor<'a> for QueryBuilderII<'a, Ta, Tb>
where
    Ta: SsqlMarker,
    Tb: SsqlMarker,
{
    fn core(&mut self) -> &mut QueryCore<'a> {
        &mut self.core
    }
}

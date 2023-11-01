use crate::structs::querybuilder::{Executable, NormalQuery};
use crate::{ColExpr, FilterExpr, QueryCore, SsqlMarker, SsqlResult};
use async_trait::async_trait;
use futures_lite::StreamExt;
use std::marker::PhantomData;
use tiberius::{Client, QueryItem, Row};
use tokio::net::TcpStream;
use tokio_util::compat::Compat;

#[async_trait]
pub trait QueryAble<'a>: Send + Sync {
    type NxtModel<NxtType: SsqlMarker>;
    type NewFnModel<NewFN, NewRet>
    where
        NewFN: Fn(&Row) -> NewRet + 'static + Send + Sync;
    type Ret;
    fn join<NxtType>(self) -> Self::NxtModel<NxtType>
    where
        NxtType: SsqlMarker;
    async fn all(&self, conn: &mut tiberius::Client<Compat<TcpStream>>) -> Self::Ret;

    fn filter(self, filter_expr: FilterExpr<'a>) -> SsqlResult<Self>
    where
        Self: Sized;

    fn replace_fn<NewFN, NewRet>(self, new_fn: NewFN) -> Self::NewFnModel<NewFN, NewRet>
    where
        NewFN: Fn(&Row) -> NewRet + 'static + Send + Sync;

    // fn order_by_asc(mut self, col_expr: ColExpr) -> SsqlResult<Self> where Self: Sized;
    // fn order_by_desc(mut self, col_expr: ColExpr) -> SsqlResult<Self> where Self: Sized;
    // async fn exec<F, Ret>(
    //     &self,
    //     conn: &mut tiberius::Client<Compat<TcpStream>>,
    //     func: F,
    // ) -> SsqlResult<Vec<Ret>>
    // where
    //     F: Fn(&tiberius::Row) -> Ret + Send + Sync,
    //     Ret: Send + Sync;
}

pub struct QueryBuilderI<'a, FN, Ret, Ta>
where
    Ta: SsqlMarker,
    FN: Fn(&Row) -> Ret + 'static + Send + Sync,
{
    pub a: QueryCore<'a, Ta>,
    pub func: FN,
}

pub struct QueryBuilderII<'a, FN, Ret, Ta, Tb>
where
    Ta: SsqlMarker,
    Tb: SsqlMarker,
    FN: Fn(&Row) -> Ret + 'static + Send + Sync,
{
    pub a: QueryCore<'a, Ta>,
    pub tb: PhantomData<Tb>,
    pub func: FN,
}

pub struct QueryBuilderIII<'a, FN, Ret, Ta, Tb, Tc>
where
    Ta: SsqlMarker,
    Tb: SsqlMarker,
    Tc: SsqlMarker,
    FN: Fn(&Row) -> Ret + 'static + Send + Sync,
{
    pub a: QueryCore<'a, Ta>,
    pub tb: PhantomData<Tb>,
    pub tc: PhantomData<Tc>,
    pub func: FN,
}
impl<'a, Ta> QueryCore<'a, Ta, NormalQuery>
where
    Ta: SsqlMarker + Send + Sync,
    QueryCore<'a, Ta, NormalQuery>: Send + Executable,
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
impl<'a, FN, Ret, Ta> QueryAble<'a> for QueryBuilderI<'a, FN, Ret, Ta>
where
    Ta: SsqlMarker + Send + Sync,
    FN: Fn(&Row) -> Ret + 'static + Send + Sync,
    // QueryCore<'a, Ta, NormalQuery>: Send + Executable,
    Ret: Send + Sync,
{
    type NxtModel<NxtType: SsqlMarker> = QueryBuilderII<'a, FN, Ret, Ta, NxtType>;
    type NewFnModel<NewFN, NewRet> = QueryBuilderI<'a, NewFN, NewRet, Ta>
    where
        NewFN: Fn(&Row) -> NewRet + 'static + Send + Sync;

    type Ret = SsqlResult<Vec<Ret>>;

    fn join<NxtType>(self) -> Self::NxtModel<NxtType>
    where
        NxtType: SsqlMarker,
    {
        QueryBuilderII {
            a: self.a.left_join::<NxtType>(),
            tb: Default::default(),
            func: self.func,
        }
    }

    async fn all(&self, conn: &mut Client<Compat<TcpStream>>) -> Self::Ret {
        self.a.exec(conn, &self.func).await
    }

    fn filter(mut self, filter_expr: FilterExpr<'a>) -> SsqlResult<Self>
    where
        Self: Sized,
    {
        self.a = self.a.filter(filter_expr)?;
        Ok(self)
    }

    fn replace_fn<NewFN, NewRet>(self, new_fn: NewFN) -> Self::NewFnModel<NewFN, NewRet>
    where
        NewFN: Fn(&Row) -> NewRet + 'static + Send + Sync,
    {
        QueryBuilderI {
            a: self.a,
            func: new_fn,
        }
    }
}

#[async_trait]
impl<'a, FN, Ret, Ta, Tb> QueryAble<'a> for QueryBuilderII<'a, FN, Ret, Ta, Tb>
where
    Ta: SsqlMarker + Send + Sync,
    Tb: SsqlMarker + Send + Sync,
    FN: Fn(&Row) -> Ret + 'static + Send + Sync,
    // QueryCore<'a, Ta, NormalQuery>: Send + Executable,
    Ret: Send + Sync,
{
    type NxtModel<NxtType: SsqlMarker> = QueryBuilderIII<'a, FN, Ret, Ta, Tb, NxtType>;
    type NewFnModel<NewFN, NewRet> = ()
    where
        NewFN: Fn(&Row) -> NewRet + 'static + Send + Sync;
    type Ret = SsqlResult<Vec<(Ret, Ret)>>;

    fn join<NxtType>(self) -> Self::NxtModel<NxtType>
    where
        NxtType: SsqlMarker,
    {
        QueryBuilderIII {
            a: self.a.left_join::<NxtType>(),
            tb: Default::default(),
            tc: Default::default(),
            func: self.func,
        }
    }

    async fn all(&self, conn: &mut Client<Compat<TcpStream>>) -> Self::Ret {
        self.a
            .exec(conn, |x| ((&self.func)(x), (&self.func)(x)))
            .await
    }

    fn filter(mut self, filter_expr: FilterExpr<'a>) -> SsqlResult<Self>
    where
        Self: Sized,
    {
        self.a = self.a.filter(filter_expr)?;
        Ok(self)
    }

    fn replace_fn<NewFN, NewRet>(self, new_fn: NewFN) -> Self::NewFnModel<NewFN, NewRet>
    where
        NewFN: Fn(&Row) -> NewRet + 'static + Send + Sync,
    {
        todo!()
    }
}
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

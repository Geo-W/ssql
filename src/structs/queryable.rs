// use crate::structs::querybuilder::{Executable, NormalQuery};
// use crate::{FilterExpr, QueryCore, SsqlMarker, SsqlResult};
// use async_trait::async_trait;
// use futures_lite::stream::StreamExt;
// use std::marker::PhantomData;
// use tiberius::{Client, QueryItem};
// use tokio::net::TcpStream;
// use tokio_util::compat::Compat;
//
// #[async_trait]
// pub trait QueryAble<'a>: Send + Sync {
//     type NxtModel<NxtType: SsqlMarker>;
//     type Ret;
//     fn join<NxtType>(self) -> Self::NxtModel<NxtType>
//     where
//         NxtType: SsqlMarker;
//     async fn all(&self, conn: &mut tiberius::Client<Compat<TcpStream>>) -> Self::Ret;
//
//     fn filter(self, filter_expr: FilterExpr<'a>) -> SsqlResult<Self>
//     where
//
//         Self: Sized;
//     // async fn exec<F, Ret>(
//     //     &self,
//     //     conn: &mut tiberius::Client<Compat<TcpStream>>,
//     //     func: F,
//     // ) -> SsqlResult<Vec<Ret>>
//     // where
//     //     F: Fn(&tiberius::Row) -> Ret + Send + Sync,
//     //     Ret: Send + Sync;
// }
//
// impl<'a, Ta> QueryCore<'a, Ta, NormalQuery>
// where
//     Ta: SsqlMarker + Send + Sync,
//     QueryCore<'a, Ta, NormalQuery>: Send + Executable,
// {
//     async fn exec<F, Ret>(
//         &self,
//         conn: &mut Client<Compat<TcpStream>>,
//         func: F,
//     ) -> SsqlResult<(Vec<Ret>)>
//     where
//         F: Fn(&tiberius::Row) -> Ret + Send + Sync,
//         Ret: Send + Sync,
//     {
//         let mut stream = self.execute(conn).await?;
//
//         let mut vec = vec![];
//
//         while let Some(item) = stream.try_next().await.unwrap() {
//             match item {
//                 QueryItem::Row(row) => vec.push(func(&row)),
//                 QueryItem::Metadata(_) => {}
//             }
//         }
//
//         Ok(vec)
//     }
// }
//
// #[async_trait]
// impl<'a, Ta> QueryAble<'a> for QueryCore<'a, Ta, NormalQuery>
// where
//     Ta: SsqlMarker + Send + Sync,
//     // QueryBuilder<'a, Ta, NormalQuery>: Send + Executable,
// {
//     type NxtModel<Tb: SsqlMarker> = QueryBuilderB<'a, Ta, NormalQuery, Tb>;
//     type Ret = SsqlResult<Vec<Ta>>;
//
//     fn join<Tb>(self) -> Self::NxtModel<Tb>
//     where
//         Tb: SsqlMarker,
//     {
//         let a = self.left_join::<Tb>();
//
//         QueryBuilderB {
//             a,
//             new: Default::default(),
//         }
//     }
//
//     async fn all(&self, conn: &mut Client<Compat<TcpStream>>) -> Self::Ret {
//         self.exec(conn, Ta::row_to_struct).await
//     }
//
//     fn filter(mut self, filter_expr: FilterExpr<'a>) -> SsqlResult<Self>
//     where
//         Self: Sized,
//     {
//         match self.tables.contains(filter_expr.col.table) {
//             true => {
//                 self.filters
//                     .push(filter_expr.to_sql(&mut self.query_idx_counter, &mut self.query_params));
//                 Ok(self)
//             }
//             false => Err("the filter applies to a table not in this builder".into()),
//         }
//     }
// }
//
// pub struct QueryBuilderB<'a, Ta, Stage, Tb>
// where
//     Ta: SsqlMarker,
//     Tb: SsqlMarker,
// {
//     a: QueryCore<'a, Ta, Stage>,
//     new: PhantomData<Tb>,
// }
//
// pub struct QueryBuilderC<'a, Ta, Stage, Tb, Tc>
// where
//     Ta: SsqlMarker,
//     Tb: SsqlMarker,
//     Tc: SsqlMarker,
// {
//     b: QueryBuilderB<'a, Ta, Stage, Tb>,
//     new: PhantomData<Tc>,
// }
//
// pub struct QueryBuilderD<'a, Ta, Stage, Tb, Tc, Td>
// where
//     Ta: SsqlMarker,
//     Tb: SsqlMarker,
//     Tc: SsqlMarker,
//     Td: SsqlMarker,
// {
//     c: QueryBuilderC<'a, Ta, Stage, Tb, Tc>,
//     new: PhantomData<Td>,
// }
//
// pub struct QueryBuilderE<'a, Ta, Stage, Tb, Tc, Td, Te>
// where
//     Ta: SsqlMarker,
//     Tb: SsqlMarker,
//     Tc: SsqlMarker,
//     Td: SsqlMarker,
//     Te: SsqlMarker,
// {
//     c: QueryBuilderD<'a, Ta, Stage, Tb, Tc, Td>,
//     new: PhantomData<Te>,
// }
//
//
// #[async_trait]
// impl<'a, Ta, Tb> QueryAble<'a> for QueryBuilderB<'a, Ta, NormalQuery, Tb>
// where
//     Ta: SsqlMarker + Send + Sync,
//     Tb: SsqlMarker + Send + Sync,
//     // QueryBuilder<'a, Ta, NormalQuery>: Send + Executable,
// {
//     type NxtModel<NxtType: SsqlMarker> = QueryBuilderC<'a, Ta, NormalQuery, Tb, NxtType>;
//     type Ret = SsqlResult<Vec<(Ta, Tb)>>;
//
//     fn join<NxtType>(mut self) -> Self::NxtModel<NxtType>
//     where
//         NxtType: SsqlMarker,
//     {
//         self.a = self.a.left_join::<NxtType>();
//         QueryBuilderC{ b: self, new: Default::default() }
//     }
//
//     async fn all(&self, conn: &mut Client<Compat<TcpStream>>) -> Self::Ret {
//         self.a
//             .exec(conn, |x| (Ta::row_to_struct(x), Tb::row_to_struct(x)))
//             .await
//     }
//
//     fn filter(mut self, filter_expr: FilterExpr<'a>) -> SsqlResult<Self> where Self: Sized {
//         self.a = self.a.filter(filter_expr)?;
//         Ok(self)
//     }
// }
//
// #[async_trait]
// impl<'a, Ta, Tb, Tc> QueryAble<'a> for QueryBuilderC<'a, Ta, NormalQuery, Tb, Tc>
// where
//     Ta: SsqlMarker + Send + Sync,
//     Tb: SsqlMarker + Send + Sync,
//     Tc: SsqlMarker + Send + Sync
// {
//     type NxtModel<NxtType: SsqlMarker> = QueryBuilderD<'a, Ta, NormalQuery, Tb, Tc, NxtType>;
//     type Ret = SsqlResult<Vec<(Ta, Tb, Tc)>>;
//
//     fn join<NxtType>(mut self) -> Self::NxtModel<NxtType> where NxtType: SsqlMarker {
//         self.b.a = self.b.a.left_join::<NxtType>();
//         QueryBuilderD{ c: self, new: Default::default() }
//     }
//
//     async fn all(&self, conn: &mut Client<Compat<TcpStream>>) -> Self::Ret {
//         self.b.a.exec(conn, |x| (Ta::row_to_struct(x), Tb::row_to_struct(x), Tc::row_to_struct(x))).await
//     }
//
//     fn filter(mut self, filter_expr: FilterExpr<'a>) -> SsqlResult<Self> where Self: Sized {
//         self.b.a = self.b.a.filter(filter_expr)?;
//         Ok(self)
//     }
// }
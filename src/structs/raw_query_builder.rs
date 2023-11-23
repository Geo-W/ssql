use crate::structs::into_result::IntoResult;
use crate::structs::query_core::{Executable, QueryCore, RawQuery};
use crate::{RowStream, SsqlMarker, SsqlResult};
use futures_lite::StreamExt;
use serde_json::Value;
use std::marker::PhantomData;
use tiberius::Client;
use tokio::net::TcpStream;
use tokio_util::compat::Compat;

pub struct RawQueryBuilder<'a, T>
where
    T: SsqlMarker,
{
    pub(crate) core: QueryCore<'a, RawQuery>,
    pub(crate) t: PhantomData<T>,
}

impl<'a, T> RawQueryBuilder<'a, T>
where
    T: SsqlMarker + Send + Sync + 'static,
{
    pub async fn all(&self, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<Vec<T>> {
        let mut stream = self.core.execute(conn).await?.into_row_stream();
        let mut ret = vec![];
        while let Some(row) = stream.try_next().await? {
            ret.push(T::to_struct(&row));
        }
        Ok(ret)
    }

    pub async fn stream<'b>(
        &self,
        conn: &'b mut tiberius::Client<Compat<TcpStream>>,
    ) -> SsqlResult<RowStream<'b, T>> {
        let stream = self.core.execute(conn).await?;
        Ok(RowStream::new(stream, T::to_struct))
    }

    pub async fn one(
        &self,
        conn: &mut tiberius::Client<Compat<TcpStream>>,
    ) -> SsqlResult<Option<T>> {
        let row = self.core.execute(conn).await?.into_row().await?;
        match row {
            None => Ok(None),
            Some(row) => Ok(Some(T::to_struct(&row))),
        }
    }

    pub async fn json(
        &self,
        conn: &mut tiberius::Client<Compat<TcpStream>>,
    ) -> SsqlResult<Vec<Value>> {
        let mut stream = self.core.execute(conn).await?.into_row_stream();
        let mut ret = vec![];
        while let Some(row) = stream.try_next().await? {
            ret.push(T::to_json(&row))
        }
        Ok(ret)
    }
}

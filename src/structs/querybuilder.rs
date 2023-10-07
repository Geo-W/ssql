use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use async_trait::async_trait;
use futures_lite::stream::StreamExt;
#[cfg(feature = "polars")]
use polars::prelude::*;
use serde_json::{Map, Value};
use tokio::net::TcpStream;
use tokio_util::compat::Compat;

use tiberius::{Client, QueryItem, QueryStream, ToSql};

use crate::error::custom_error::{SsqlError, SsqlResult};
use crate::structs::filter::{ColExpr, FilterExpr};

pub struct QueryBuilder<'a, T: SsqlMarker> {
    pub(crate) fields: HashMap<&'static str, Vec<&'static str>>,
    pub(crate) filters: Vec<String>,
    pub(crate) join: String,
    tables: HashSet<&'static str>,
    raw_sql: Option<String>,
    relation_func: fn(&str) -> &'static str,
    query_params: Vec<&'a dyn ToSql>,
    query_idx_counter: i32,

    _marker: Option<PhantomData<T>>,
}

impl<'a, T> QueryBuilder<'a, T>
    where T: SsqlMarker
{
    pub fn new<'b : 'a, C>(fields: (&'static str, Vec<&'static str>), func: fn(&str) -> &'static str) -> QueryBuilder<'b, C>
        where C: SsqlMarker
    {
        QueryBuilder {
            tables: HashSet::from([fields.0]),
            fields: HashMap::from([fields]),
            filters: vec![],
            join: String::new(),
            relation_func: func,
            raw_sql: None,
            _marker: None,
            query_params: vec![],
            query_idx_counter: 0,
        }
    }

    /// Chain a filter to current builder.
    /// This method will check whether the table provided is in this builder thus [`SsqlResult`] is returned.
    pub fn filter(mut self, filter_expr: FilterExpr<'a>) -> SsqlResult<Self> {
        // self.query_params.push(filter_expr.conditions);
        match self.tables.contains(filter_expr.col.table) {
            true => {
                self.filters.push(filter_expr.to_sql(&mut self.query_idx_counter, &mut self.query_params));
                Ok(self)
            }
            false => {
                Err(SsqlError::new("the filter applies to a table not in this builder"))
            }
        }
    }

    fn join<B>(mut self, join_type: &str) -> QueryBuilder<'a, T>
        where B: SsqlMarker {
        let name = B::table_name();
        let fields = B::fields();
        let relation = self.find_relation(&name);
        self.join.push_str(&format!(" {} JOIN {} ", join_type, relation));
        match self.fields.insert(&name, fields) {
            Some(_v) => panic!("table already joined."),
            None => {
                self.tables.insert(name);
            }
        }
        self
    }

    pub fn left_join<B>(self) -> QueryBuilder<'a, T>
        where B: SsqlMarker {
        self.join::<B>("LEFT")
    }

    pub fn right_join<B>(self) -> QueryBuilder<'a, T>
        where B: SsqlMarker {
        self.join::<B>("RIGHT")
    }

    pub fn inner_join<B>(self) -> QueryBuilder<'a, T>
        where B: SsqlMarker {
        self.join::<B>("INNER")
    }

    pub fn outer_join<B>(self) -> QueryBuilder<'a, T>
        where B: SsqlMarker {
        self.join::<B>("OUTER")
    }

    fn find_relation(&self, table: &str) -> &'static str {
        (self.relation_func)(table)
    }

    pub fn raw<'b: 'a>(mut self, sql: impl ToString, params: &[&'b dyn ToSql]) -> Self {
        self.raw_sql = Some(sql.to_string());
        for p in params {
            self.query_params.push(*p);
        }
        self
    }

    crate::impl_get_data!(get_serialized, row_to_json, [A, ret1, Value]);
    crate::impl_get_data!(get_serialized_2, row_to_json, [A, ret1, Value, B, ret2, Value]);
    crate::impl_get_data!(get_serialized_3, row_to_json, [A, ret1, Value, B, ret2, Value, C, ret3, Value]);
    crate::impl_get_data!(get_serialized_4, row_to_json, [A, ret1, Value, B, ret2, Value, C, ret3, Value, D, ret4, Value]);
    crate::impl_get_data!(get_serialized_5, row_to_json, [A, ret1, Value, B, ret2, Value, C, ret3, Value, D, ret4, Value, E, ret5, Value]);

    crate::impl_get_data!(get_struct, row_to_struct, [A, ret1, A]);
    crate::impl_get_data!(get_struct_2, row_to_struct, [A, ret1, A, B, ret2, B]);
    crate::impl_get_data!(get_struct_3, row_to_struct, [A, ret1, A, B, ret2, B, C, ret3, C]);
    crate::impl_get_data!(get_struct_4, row_to_struct, [A, ret1, A, B, ret2, B, C, ret3, C, D, ret4, D]);
    crate::impl_get_data!(get_struct_5, row_to_struct, [A, ret1, A, B, ret2, B, C, ret3, C, D, ret4, D, E, ret5, E]);

    crate::impl_get_dataframe!(get_dataframe, get_struct, [A, ret1, DataFrame]);
    crate::impl_get_dataframe!(get_dataframe_2, get_struct_2, [A, ret1, DataFrame, B, ret2, DataFrame]);
    crate::impl_get_dataframe!(get_dataframe_3, get_struct_3, [A, ret1, DataFrame, B, ret2, DataFrame, C, ret3, DataFrame]);
    crate::impl_get_dataframe!(get_dataframe_4, get_struct_4, [A, ret1, DataFrame, B, ret2, DataFrame, C, ret3, DataFrame, D, ret4, DataFrame]);
    crate::impl_get_dataframe!(get_dataframe_5, get_struct_5, [A, ret1, DataFrame, B, ret2, DataFrame, C, ret3, DataFrame, D, ret4, DataFrame, E, ret5, DataFrame]);


    async fn execute<'b>(&self, conn: &'b mut tiberius::Client<Compat<TcpStream>>) -> SsqlResult<QueryStream<'b>> {
        let select_fields = self.fields.iter()
            .map(|(table, fields)|
                fields.iter().map(|field| format!(r#"{}.{} as "{}.{}""#, table, field, table, field))
                    .reduce(|cur, nxt| format!("{},{}", cur, nxt)).unwrap()
            )
            .reduce(|cur, nxt| format!("{},{}", cur, nxt)).unwrap();

        let where_clause = self.get_where_clause();
        dbg!(&where_clause);

        // let mut stream = conn.simple_query(r#"SELECT ship_to_id as "CUSTOMER_LIST.ship_to_id", ship_to as "CUSTOMER_LIST.ship_to",
        // volume as "CUSTOMER_LIST.volume", container as "CUSTOMER_LIST.container" FROM CUSTOMER_LIST"#).await.unwrap();
        let stream = conn.query(match &self.raw_sql {
            None => {
                format!("SELECT {} FROM {} {} {where_clause}", select_fields, T::table_name(), self.join)
            }
            Some(v) => {
                v.to_string()
            }
        }, self.query_params.as_slice()).await?;
        Ok(stream)
    }

    pub async fn delete(dt: &dyn ToSql, pk: &'static str, conn: &mut tiberius::Client<Compat<TcpStream>>) -> SsqlResult<()> {
        conn.execute(format!("DELETE FROM {} WHERE {} = @p1", T::table_name(), pk), &[dt]).await?;
        Ok(())
    }

    fn get_where_clause(&self) -> String {
        match self.filters.iter()
            .map(|x| x.clone())
            .reduce(|cur, nxt| format!("{} AND {}", cur, nxt))
        {
            None => "".to_string(),
            Some(v) => format!(" WHERE {}", v)
        }
    }
}


/// a trait
#[async_trait(? Send)]
pub trait SsqlMarker: Sized {
    fn table_name() -> &'static str;
    fn fields() -> Vec<&'static str>;
    fn row_to_json(row: &tiberius::Row) -> Map<String, Value>;
    fn row_to_struct(row: &tiberius::Row) -> Self;
    fn query<'a>() -> QueryBuilder<'a, Self>;
    async fn insert_many(iter: impl IntoIterator<Item=Self>, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<u64>;
    async fn insert(self, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<()>;
    async fn delete(self, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<()>;
    async fn update(&self, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<()>;

    fn col(field: &'static str) -> SsqlResult<ColExpr> {
        match Self::fields().contains(&field) {
            true => {
                Ok(ColExpr { table: Self::table_name(), field })
            }
            false => {
                Err(SsqlError::new(format!("column {} not found in {}", field, Self::table_name())))
            }
        }
    }
}

#[cfg(feature = "polars")]
pub trait PolarsHelper {
    fn dataframe(vec: Vec<Self>) -> PolarsResult<DataFrame>
        where Self: Sized;
}

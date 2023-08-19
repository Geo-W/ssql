use std::collections::HashMap;
use std::marker::PhantomData;

use async_trait::async_trait;
use futures_lite::stream::StreamExt;
#[cfg(feature = "polars")]
use polars::prelude::*;
use serde_json::{Map, Value};
use tokio::net::TcpStream;
use tokio_util::compat::Compat;

use tiberius::{Client, ColumnData, QueryItem, QueryStream};

use crate::error::custom_error::RssqlResult;

pub struct QueryBuilder<T: RusqlMarker> {
    table: &'static str,
    pub(crate) fields: HashMap<&'static str, Vec<&'static str>>,
    pub(crate) filters: Vec<String>,
    pub(crate) join: String,
    tables: Vec<&'static str>,
    // pub query_result: HashMap<&'static str, Vec<Value>>,
    sql: String,
    // relation_func: Box<dyn Fn(&str) -> &'static str>,
    relation_func: fn(&str) -> &'static str,

    row_to_json_func: HashMap<String, Box<dyn Fn(&tiberius::Row) -> Map<String, Value> + Send + 'static>>,
    _marker: Option<PhantomData<T>>
    // mapper from table name to select row func

}

impl<T> QueryBuilder<T>
where T: RusqlMarker
{
    pub fn new<C>(table: &'static str, fields: (&'static str, Vec<&'static str>), func: fn(&str) -> &'static str, row_to_json: Box<dyn Fn(&tiberius::Row) -> Map<String, Value> + Send + 'static>) -> QueryBuilder<C>
    where C: RusqlMarker
    {
        QueryBuilder {
            table: table,
            fields: HashMap::from([fields]),
            filters: vec![],
            tables: vec![table],
            join: String::new(),
            relation_func: func,
            sql: "".to_string(),
            row_to_json_func: HashMap::from([(table.to_string(), row_to_json)]),
            _marker: None
        }
    }

    pub fn filter(mut self, field: &str, condition: impl ToString) -> Self {
        self.filters.push(format!("{}{}", field, condition.to_string()));
        self
    }

    pub fn join<B>(mut self) -> QueryBuilder<T>
        where B: RusqlMarker + 'static {
        let name = B::table_name();
        let fields = B::fields();
        println!("name: {:?}", name);
        let relation = self.find_relation(&name);
        self.join.push_str(&format!(" LEFT JOIN {} ", relation));
        match self.fields.insert(&name, fields) {
            Some(_v) => panic!("table already joined."),
            None => {
                self.tables.push(name);
                self.row_to_json_func.insert(name.to_string(), Box::new(B::row_to_json));
            }
        }
        self
    }

    fn find_relation(&self, table: &str) -> &'static str {
        (self.relation_func)(table)
    }

    pub fn raw(mut self, sql: &str) -> Self {
        self.sql = sql.to_string();
        self
    }

    pub async fn find_all(&mut self, _conn: &mut tiberius::Client<Compat<TcpStream>>) -> RssqlResult<()> {
        Ok(())
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



    #[cfg(feature = "polars")]
    pub async fn get_dataframe<A>(&mut self, conn: &mut tiberius::Client<Compat<TcpStream>>) -> RssqlResult<(DataFrame)>
        where A: RusqlMarker + PolarsHelper + std::fmt::Debug
    {
        let vec1 = self.get_self(conn).await?;
        Ok(A::dataframe(vec1)?)
    }

    #[cfg(feature = "polars")]
    pub async fn get_dataframe_2<A, B>(&mut self, conn: &mut tiberius::Client<Compat<TcpStream>>) -> RssqlResult<(DataFrame, DataFrame)>
        where A: RusqlMarker + PolarsHelper + std::fmt::Debug,
              B: RusqlMarker + PolarsHelper + std::fmt::Debug
    {
        let (vec1, vec2) = self.get_self_2::<A, B>(conn).await?;
        Ok((A::dataframe(vec1)?, B::dataframe(vec2)?))
    }

    async fn execute<'a>(&mut self, conn: &'a mut tiberius::Client<Compat<TcpStream>>) -> RssqlResult<QueryStream<'a>> {
        let sql = self.fields.iter()
            .map(|(table, fields)|
                fields.iter().map(|field| format!(r#"{}.{} as "{}.{}""#, table, field, table, field))
                    .reduce(|cur, nxt| format!("{},{}", cur, nxt)).unwrap()
            )
            .reduce(|cur, nxt| format!("{},{}", cur, nxt)).unwrap();

        // let mut stream = conn.simple_query(r#"SELECT ship_to_id as "CUSTOMER_LIST.ship_to_id", ship_to as "CUSTOMER_LIST.ship_to",
        // volume as "CUSTOMER_LIST.volume", container as "CUSTOMER_LIST.container" FROM CUSTOMER_LIST"#).await.unwrap();
        dbg!(format!("SELECT {} FROM {} {} ", sql, self.table, self.join));
        let stream = conn.simple_query(format!("SELECT {} FROM {} {} ", sql, self.table, self.join)).await?;
        Ok(stream)
    }

    pub fn process_pk_condition(dt: &ColumnData<'_>) -> String {
        match dt {
            ColumnData::U8(v) => {
                match v {
                    None => { " is null ".to_string() }
                    Some(v) => {
                        format!(" = {} ", v)
                    }
                }
            }
            ColumnData::I32(v) => {
                match v {
                    None => { " is null ".to_string() }
                    Some(v) => {
                        format!(" = {} ", v)
                    }
                }
            }
            ColumnData::I64(v) => {
                match v {
                    None => { " is null ".to_string() }
                    Some(v) => {
                        format!(" = {} ", v)
                    }
                }
            }
            _ => {
                unimplemented!()
            }
        }
    }

    pub async fn delete(dt: &ColumnData<'_>, table: &'static str, pk: &'static str, conn: &mut tiberius::Client<Compat<TcpStream>>) -> RssqlResult<()> {
        let condition = Self::process_pk_condition(&dt);
        conn.execute(format!("DELETE FROM {} WHERE {} {}", table, pk, condition), &[]).await?;
        Ok(())
    }
}


#[async_trait(? Send)]
pub trait RusqlMarker: Sized {
    fn table_name() -> &'static str;
    fn fields() -> Vec<&'static str>;
    fn row_to_json(row: &tiberius::Row) -> Map<String, Value>;
    fn row_to_struct(row: &tiberius::Row) -> Self;
    async fn insert_many(iter: impl IntoIterator<Item=Self>, conn: &mut Client<Compat<TcpStream>>) -> RssqlResult<u64>;
    async fn insert_one(self, conn: &mut Client<Compat<TcpStream>>) -> RssqlResult<()>;
    async fn delete(self, conn: &mut Client<Compat<TcpStream>>) -> RssqlResult<()>;
    async fn update(&self, conn: &mut Client<Compat<TcpStream>>) -> RssqlResult<()>;
}

#[cfg(feature = "polars")]
pub trait PolarsHelper {
    fn dataframe(vec: Vec<Self>) -> PolarsResult<DataFrame>
        where Self: Sized;
}

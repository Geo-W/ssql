use std::collections::HashMap;

use async_trait::async_trait;
use futures_lite::stream::StreamExt;
#[cfg(feature = "polars")]
use polars::prelude::*;
use serde_json::{Map, Value};
use tokio::net::TcpStream;
use tokio_util::compat::Compat;

use tiberius::{Client, Column, ColumnData, QueryItem, QueryStream};

use crate::error::custom_error::RssqlResult;

pub struct QueryBuilder {
    table: &'static str,
    pub(crate) fields: HashMap<&'static str, Vec<&'static str>>,
    pub(crate) filters: Vec<String>,
    pub(crate) join: String,
    tables: Vec<&'static str>,
    pub query_result: HashMap<&'static str, Vec<Value>>,
    sql: String,
    // relation_func: Box<dyn Fn(&str) -> &'static str>,
    relation_func: fn(&str) -> &'static str,

    row_to_json_func: HashMap<String, Box<dyn Fn(&tiberius::Row) -> Map<String, Value> + Send + 'static>>,
    // mapper from table name to select row func

    // #[cfg(feature = "polars")]
}

impl QueryBuilder {
    pub fn new(table: &'static str, fields: (&'static str, Vec<&'static str>), func: fn(&str) -> &'static str, row_to_json: Box<dyn Fn(&tiberius::Row) -> Map<String, Value> + Send + 'static>) -> Self {
        QueryBuilder {
            table: table,
            fields: HashMap::from([fields]),
            filters: vec![],
            tables: vec![table],
            join: String::new(),
            relation_func: func,
            sql: "".to_string(),
            row_to_json_func: HashMap::from([(table.to_string(), row_to_json)]),
            query_result: HashMap::from([(table, vec![])]),
        }
    }

    pub fn filter(mut self, field: &str, condition: impl ToString) -> Self {
        self.filters.push(format!("{}{}", field, condition.to_string()));
        self
    }

    pub fn join<T>(mut self) -> QueryBuilder
        where T: RusqlMarker + 'static {
        let name = T::table_name();
        let fields = T::fields();
        println!("name: {:?}", name);
        let relation = self.find_relation(&name);
        self.join.push_str(&format!(" LEFT JOIN {} ", relation));
        match self.fields.insert(&name, fields) {
            Some(_v) => panic!("table already joined."),
            None => {
                self.tables.push(name);
                self.query_result.insert(name, vec![]);
                self.row_to_json_func.insert(name.to_string(), Box::new(T::row_to_json));
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

    pub async fn find_all(&mut self, conn: &mut tiberius::Client<Compat<TcpStream>>) -> RssqlResult<()> {
        let sql = self.fields.iter()
            .map(|(table, fields)|
                fields.iter().map(|field| format!(r#"{}.{} as "{}.{}""#, table, field, table, field))
                    .reduce(|cur, nxt| format!("{},{}", cur, nxt)).unwrap()
            )
            .reduce(|cur, nxt| format!("{},{}", cur, nxt)).unwrap();

        // let mut stream = conn.simple_query(r#"SELECT ship_to_id as "CUSTOMER_LIST.ship_to_id", ship_to as "CUSTOMER_LIST.ship_to",
        // volume as "CUSTOMER_LIST.volume", container as "CUSTOMER_LIST.container" FROM CUSTOMER_LIST"#).await.unwrap();
        dbg!(format!("SELECT {} FROM {} {} ", sql, self.table, self.join));
        let mut stream = conn.simple_query(format!("SELECT {} FROM {} {} ", sql, self.table, self.join)).await?;
        while let Some(item) = stream.try_next().await.unwrap() {
            match item {
                QueryItem::Row(row) => {
                    self.tables.iter().for_each(|table| {
                        self.query_result.get_mut(table).unwrap().push(
                            (self.row_to_json_func.get(&table.to_string()).unwrap())(&row).into());
                    });
                }
                QueryItem::Metadata(_) => {}
            }
        }
        Ok(())
    }


    crate::impl_get_self!(get_self, [A, ret1]);
    crate::impl_get_self!(get_self_2, [A, ret1, B, ret2]);
    crate::impl_get_self!(get_self_3, [A, ret1, B, ret2, C, ret3]);



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


    pub fn get<B: RusqlMarker>(&mut self) -> Vec<Value> {
        self.query_result.remove(B::table_name()).unwrap()
    }

    pub async fn delete(dt: &ColumnData<'_>, table: &'static str, pk: &'static str, conn: &mut tiberius::Client<Compat<TcpStream>>) -> RssqlResult<()> {
        let condition = match dt {
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
        };

        conn.execute(format!("DELETE FROM {} WHERE {} {}", table, pk, condition), &[]).await?;
        Ok(())
    }
}


#[async_trait(? Send)]
pub trait RusqlMarker: Sized {
    fn table_name() -> &'static str;
    fn fields() -> Vec<&'static str>;
    fn row_to_json(row: &tiberius::Row) -> Map<String, Value>;
    fn row_to_self(row: &tiberius::Row) -> Self;
    async fn insert_many(iter: impl IntoIterator<Item=Self>, conn: &mut Client<Compat<TcpStream>>) -> RssqlResult<u64>;
    async fn insert_one(self, conn: &mut Client<Compat<TcpStream>>) -> RssqlResult<()>;
    async fn delete(self, conn: &mut Client<Compat<TcpStream>>) -> RssqlResult<()>;
}

#[cfg(feature = "polars")]
pub trait PolarsHelper {
    fn dataframe(vec: Vec<Self>) -> PolarsResult<DataFrame>
        where Self: Sized;
}

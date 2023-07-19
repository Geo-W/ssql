use std::collections::HashMap;
use serde_json::{Map, Value};
use tiberius::{AuthMethod, Client, Config, QueryItem};
use tokio::net::TcpStream;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};
use futures_lite::stream::StreamExt;
use async_trait::async_trait;

use crate::error::custom_error::RssqlError;

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

    from_row_funcs: HashMap<String, Box<dyn Fn(&tiberius::Row) -> Map<String, Value>>>, // mapper from table name to select row func
}

impl QueryBuilder {
    pub fn new(table: &'static str, fields: (&'static str, Vec<&'static str>), func: fn(&str) -> &'static str, from_row: Box<dyn Fn(&tiberius::Row) -> Map<String, Value>>) -> Self {
        QueryBuilder {
            table: table,
            fields: HashMap::from([fields]),
            filters: vec![],
            tables: vec![table],
            join: String::new(),
            relation_func: func,
            sql: "".to_string(),
            from_row_funcs: HashMap::from([(table.to_string(), from_row)]),
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
                self.from_row_funcs.insert(name.to_string(), Box::new(T::from_row));
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

    pub async fn find_all(&mut self, mut conn: tiberius::Client<Compat<TcpStream>>) -> Result<(), RssqlError> {
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
                    dbg!(&row);
                    self.tables.iter().for_each(|table| {
                        let vec = self.query_result.get_mut(table).unwrap().push(
                            (self.from_row_funcs.get(&table.to_string()).unwrap())(&row).into());
                    });
                    // ret.push((self.from_row_funcs.get(self.table).unwrap())(&row).into());
                }
                QueryItem::Metadata(_) => {}
            }
        }
        Ok(())
    }

    pub fn get<T: RusqlMarker>(&mut self) -> Vec<Value> {
        self.query_result.remove(T::table_name()).unwrap()
    }

}

#[async_trait(?Send)]
pub trait RusqlMarker: Sized {
    fn table_name() -> &'static str;
    fn fields() -> Vec<&'static str>;
    fn from_row(row: &tiberius::Row) -> Map<String, Value>;
    async fn insert_many(iter: impl Iterator<Item = Self>, conn: Client<Compat<TcpStream>>) -> Result<u64, RssqlError>;
}

struct TableInfo<RusqlMarker> {
    table_name: String,
    relationship: Box<dyn Fn(&str) -> &'static str>,
    from_row: Box<dyn Fn(tiberius::Row) -> RusqlMarker>,
}
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use async_trait::async_trait;
use futures_lite::stream::StreamExt;
#[cfg(feature = "polars")]
use polars::prelude::*;
use serde_json::{Map, Value};
use tokio::net::TcpStream;
use tokio_util::compat::Compat;

use tiberius::{Client, ColumnData, QueryItem, QueryStream};

use crate::error::custom_error::SsqlResult;

pub struct QueryBuilder<T: SsqlMarker> {
    pub(crate) fields: HashMap<&'static str, Vec<&'static str>>,
    pub(crate) filters: Vec<String>,
    pub(crate) join: String,
    tables: HashSet<&'static str>,
    // pub query_result: HashMap<&'static str, Vec<Value>>,
    sql: String,
    // relation_func: Box<dyn Fn(&str) -> &'static str>,
    relation_func: fn(&str) -> &'static str,

    _marker: Option<PhantomData<T>>,
    // mapper from table name to select row func
}

impl<T> QueryBuilder<T>
    where T: SsqlMarker
{
    pub fn new<C>(fields: (&'static str, Vec<&'static str>), func: fn(&str) -> &'static str) -> QueryBuilder<C>
        where C: SsqlMarker
    {
        QueryBuilder {
            fields: HashMap::from([fields]),
            filters: vec![],
            tables: HashSet::new(),
            join: String::new(),
            relation_func: func,
            sql: "".to_string(),
            _marker: None,
        }
    }

    pub fn filter(mut self, field: &str, condition: impl ToString) -> Self {
        self.filters.push(format!("{}{}", field, condition.to_string()));
        self
    }

    fn join<B>(mut self, join_type: &str) -> QueryBuilder<T>
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

    pub fn left_join<B>(self) -> QueryBuilder<T>
        where B: SsqlMarker {
        self.join::<B>("LEFT")
    }

    pub fn right_join<B>(self) -> QueryBuilder<T>
        where B: SsqlMarker {
        self.join::<B>("RIGHT")
    }

    pub fn inner_join<B>(self) -> QueryBuilder<T>
        where B: SsqlMarker {
        self.join::<B>("INNER")
    }

    pub fn outer_join<B>(self) -> QueryBuilder<T>
        where B: SsqlMarker {
        self.join::<B>("OUTER")
    }

    fn find_relation(&self, table: &str) -> &'static str {
        (self.relation_func)(table)
    }

    pub fn raw(mut self, sql: &str) -> Self {
        self.sql = sql.to_string();
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


    async fn execute<'a>(&mut self, conn: &'a mut tiberius::Client<Compat<TcpStream>>) -> SsqlResult<QueryStream<'a>> {
        let sql = self.fields.iter()
            .map(|(table, fields)|
                fields.iter().map(|field| format!(r#"{}.{} as "{}.{}""#, table, field, table, field))
                    .reduce(|cur, nxt| format!("{},{}", cur, nxt)).unwrap()
            )
            .reduce(|cur, nxt| format!("{},{}", cur, nxt)).unwrap();

        // let mut stream = conn.simple_query(r#"SELECT ship_to_id as "CUSTOMER_LIST.ship_to_id", ship_to as "CUSTOMER_LIST.ship_to",
        // volume as "CUSTOMER_LIST.volume", container as "CUSTOMER_LIST.container" FROM CUSTOMER_LIST"#).await.unwrap();
        dbg!(format!("SELECT {} FROM {} {} ", sql, T::table_name(), self.join));
        let stream = conn.simple_query(format!("SELECT {} FROM {} {} ", sql, T::table_name(), self.join)).await?;
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
            ColumnData::I16(v) => {
                match v {
                    None => { " is null ".to_string() }
                    Some(v) => {
                        format!(" = {} ", v)
                    }
                }
            }
            ColumnData::F32(v) => {
                match v {
                    None => { " is null ".to_string() }
                    Some(v) => {
                        format!(" = {} ", v)
                    }
                }
            }
            ColumnData::F64(v) => {
                match v {
                    None => { " is null ".to_string() }
                    Some(v) => {
                        format!(" = {} ", v)
                    }
                }
            }
            ColumnData::Bit(v) => {
                match v {
                    None => { " is null ".to_string() }
                    Some(v) => {
                        format!(" = {} ", v)
                    }
                }
            }
            ColumnData::String(v) => {
                match v {
                    None => { " is null ".to_string() }
                    Some(v) => {
                        format!(" = {} ", v)
                    }
                }
            }
            ColumnData::Guid(v) => {
                match v {
                    None => { " is null ".to_string() }
                    Some(v) => {
                        format!(" = {} ", v)
                    }
                }
            }
            ColumnData::Binary(v) => {
                match v {
                    None => { " is null ".to_string() }
                    Some(v) => {
                        format!(" = {} ", String::from_utf8(v.to_vec()).unwrap())
                    }
                }
            }
            ColumnData::Numeric(v) => {
                match v {
                    None => { " is null ".to_string() }
                    Some(v) => {
                        format!(" = {} ", v)
                    }
                }
            }
            ColumnData::Xml(v) => {
                match v {
                    None => { " is null ".to_string() }
                    Some(v) => {
                        "TODO".to_string()
                        // TODO!
                    }
                }
            }
            ColumnData::DateTime(v) => {
                match v {
                    None => { " is null ".to_string() }
                    Some(v) => {
                        "TODO".to_string()
                        // TODO!
                    }
                }
            }
            ColumnData::SmallDateTime(v) => {
                match v {
                    None => { " is null ".to_string() }
                    Some(v) => {
                        "TODO".to_string()
                        // TODO!
                    }
                }
            }
            ColumnData::Time(v) => {
                match v {
                    None => { " is null ".to_string() }
                    Some(v) => {
                        "TODO".to_string()
                        // TODO!
                    }
                }
            }
            ColumnData::Date(v) => {
                match v {
                    None => { " is null ".to_string() }
                    Some(v) => {
                        "TODO".to_string()
                        // TODO!
                    }
                }
            }
            ColumnData::DateTime2(v) => {
                match v {
                    None => { " is null ".to_string() }
                    Some(v) => {
                        "TODO".to_string()
                        // TODO!
                    }
                }
            }
            ColumnData::DateTimeOffset(v) => {
                match v {
                    None => { " is null ".to_string() }
                    Some(v) => {
                        "TODO".to_string()
                        // TODO!
                    }
                }
            }
        }
    }

    pub async fn delete(dt: &ColumnData<'_>, table: &'static str, pk: &'static str, conn: &mut tiberius::Client<Compat<TcpStream>>) -> SsqlResult<()> {
        let condition = Self::process_pk_condition(&dt);
        conn.execute(format!("DELETE FROM {} WHERE {} {}", table, pk, condition), &[]).await?;
        Ok(())
    }
}


#[async_trait(? Send)]
pub trait SsqlMarker: Sized {
    fn table_name() -> &'static str;
    fn fields() -> Vec<&'static str>;
    fn row_to_json(row: &tiberius::Row) -> Map<String, Value>;
    fn row_to_struct(row: &tiberius::Row) -> Self;
    async fn insert_many(iter: impl IntoIterator<Item=Self>, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<u64>;
    async fn insert(self, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<()>;
    async fn delete(self, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<()>;
    async fn update(&self, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<()>;
}

#[cfg(feature = "polars")]
pub trait PolarsHelper {
    fn dataframe(vec: Vec<Self>) -> PolarsResult<DataFrame>
        where Self: Sized;
}

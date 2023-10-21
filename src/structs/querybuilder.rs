use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use async_trait::async_trait;
use futures_lite::stream::StreamExt;
#[cfg(feature = "polars")]
use polars::prelude::*;
use serde_json::{Map, Value};
use tiberius::{Client, QueryItem, QueryStream, ToSql};
use tokio::net::TcpStream;
use tokio_util::compat::Compat;

use crate::error::custom_error::SsqlResult;
use crate::structs::filter::{ColExpr, FilterExpr};

pub struct RawQuery;

pub struct NormalQuery;

#[async_trait]
pub trait Executable {
    async fn execute<'b>(&self, conn: &'b mut tiberius::Client<Compat<TcpStream>>) -> SsqlResult<QueryStream<'b>>;
}

#[async_trait]
impl<'a, T> Executable for QueryBuilder<'a, T, NormalQuery>
    where T: SsqlMarker + Send + Sync
{
    async fn execute<'b>(&self, conn: &'b mut tiberius::Client<Compat<TcpStream>>) -> SsqlResult<QueryStream<'b>> {
        let select_fields = self.fields.iter()
            .map(|(table, fields)|
                fields.iter().map(|field| format!(r#"{}.{} as "{}.{}""#, table, field, table, field))
                    .reduce(|cur, nxt| format!("{},{}", cur, nxt)).unwrap()
            )
            .reduce(|cur, nxt| format!("{},{}", cur, nxt)).unwrap();

        let where_clause = self.get_where_clause();

        // let mut stream = conn.simple_query(r#"SELECT ship_to_id as "CUSTOMER_LIST.ship_to_id", ship_to as "CUSTOMER_LIST.ship_to",
        // volume as "CUSTOMER_LIST.volume", container as "CUSTOMER_LIST.container" FROM CUSTOMER_LIST"#).await.unwrap();
        let stream = conn.query(
            format!("SELECT {} FROM {} {} {where_clause}", select_fields, T::table_name(), self.join),
            self.query_params.as_slice(),
        ).await?;
        Ok(stream)
    }
}

#[async_trait]
impl<'a, T> Executable for QueryBuilder<'a, T, RawQuery>
    where T: SsqlMarker + Send + Sync
{
    async fn execute<'b>(&self, conn: &'b mut tiberius::Client<Compat<TcpStream>>) -> SsqlResult<QueryStream<'b>> {
        let stream = conn.query(self.raw_sql.as_ref().unwrap(),
                                self.query_params.as_slice()).await?;
        Ok(stream)
    }
}

/// Query object generated by [`TableStruct::query()`], for constructing a builder, making a query, etc.
///
/// [`TableStruct::query()`]: trait.SsqlMarker.html#tymethod.query
pub struct QueryBuilder<'a, T: SsqlMarker, Stage = NormalQuery> {
    pub(crate) fields: HashMap<&'static str, Vec<&'static str>>,
    pub(crate) filters: Vec<String>,
    pub(crate) join: String,
    tables: HashSet<&'static str>,
    raw_sql: Option<String>,
    relation_func: fn(&str) -> &'static str,
    query_params: Vec<&'a dyn ToSql>,
    query_idx_counter: i32,

    _marker: Option<PhantomData<T>>,
    _mark2: PhantomData<Stage>,
}

impl<'a, T, Stage: 'static> QueryBuilder<'a, T, Stage>
    where T: SsqlMarker,
          QueryBuilder<'a, T, Stage>: Executable
{
    impl_get_data!(get_serialized, row_to_json, [A, ret1, Value]);
    impl_get_data!(get_serialized_2, row_to_json, [A, ret1, Value, B, ret2, Value]);
    impl_get_data!(get_serialized_3, row_to_json, [A, ret1, Value, B, ret2, Value, C, ret3, Value]);
    impl_get_data!(get_serialized_4, row_to_json, [A, ret1, Value, B, ret2, Value, C, ret3, Value, D, ret4, Value]);
    impl_get_data!(get_serialized_5, row_to_json, [A, ret1, Value, B, ret2, Value, C, ret3, Value, D, ret4, Value, E, ret5, Value]);

    impl_get_data!(get_struct, row_to_struct, [A, ret1, A]);
    impl_get_data!(get_struct_2, row_to_struct, [A, ret1, A, B, ret2, B]);
    impl_get_data!(get_struct_3, row_to_struct, [A, ret1, A, B, ret2, B, C, ret3, C]);
    impl_get_data!(get_struct_4, row_to_struct, [A, ret1, A, B, ret2, B, C, ret3, C, D, ret4, D]);
    impl_get_data!(get_struct_5, row_to_struct, [A, ret1, A, B, ret2, B, C, ret3, C, D, ret4, D, E, ret5, E]);

    impl_get_dataframe!(get_dataframe, get_struct, [A, ret1, DataFrame]);
    impl_get_dataframe!(get_dataframe_2, get_struct_2, [A, ret1, DataFrame, B, ret2, DataFrame]);
    impl_get_dataframe!(get_dataframe_3, get_struct_3, [A, ret1, DataFrame, B, ret2, DataFrame, C, ret3, DataFrame]);
    impl_get_dataframe!(get_dataframe_4, get_struct_4, [A, ret1, DataFrame, B, ret2, DataFrame, C, ret3, DataFrame, D, ret4, DataFrame]);
    impl_get_dataframe!(get_dataframe_5, get_struct_5, [A, ret1, DataFrame, B, ret2, DataFrame, C, ret3, DataFrame, D, ret4, DataFrame, E, ret5, DataFrame]);
}

impl<'a, T> QueryBuilder<'a, T, NormalQuery>
    where T: SsqlMarker
{
    #[doc(hidden)]
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
            _mark2: PhantomData,
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
                Err("the filter applies to a table not in this builder".into())
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
    pub fn left_join<B>(self) -> QueryBuilder<'a, T>
        where B: SsqlMarker {
        self.join::<B>("LEFT")
    }

    /// Perform right join on another table.
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
    /// let _ = Person::query().right_join::<Posts>();
    /// ```
    /// SQL: `... FROM SCHEMA1.person RIGHT JOIN posts ON SCHEMA1.Person.id = posts.person_id`
    pub fn right_join<B>(self) -> QueryBuilder<'a, T>
        where B: SsqlMarker {
        self.join::<B>("RIGHT")
    }

    /// Perform inner join on another table.
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
    /// let _ = Person::query().inner_join::<Posts>();
    /// ```
    /// SQL: `... FROM SCHEMA1.person INNER JOIN posts ON SCHEMA1.Person.id = posts.person_id`
    pub fn inner_join<B>(self) -> QueryBuilder<'a, T>
        where B: SsqlMarker {
        self.join::<B>("INNER")
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
    /// let _ = Person::query().outer_join::<Posts>();
    /// ```
    /// SQL: `... FROM SCHEMA1.person OUTER JOIN posts ON SCHEMA1.Person.id = posts.person_id`
    pub fn outer_join<B>(self) -> QueryBuilder<'a, T>
        where B: SsqlMarker {
        self.join::<B>("OUTER")
    }

    fn find_relation(&self, table: &str) -> &'static str {
        (self.relation_func)(table)
    }

    fn get_where_clause(&self) -> String {
        match self.filters.iter().cloned()
            .reduce(|cur, nxt| format!("{} AND {}", cur, nxt))
        {
            None => "".to_string(),
            Some(v) => format!(" WHERE {}", v)
        }
    }
}


/// a trait automatically derived via `#[derive(ORM)]` macro, all these methods are available.
#[async_trait]
pub trait SsqlMarker {
    #[doc(hidden)]
    fn table_name() -> &'static str where Self: Sized;
    #[doc(hidden)]
    fn fields() -> Vec<&'static str> where Self: Sized;
    #[doc(hidden)]
    fn row_to_json(row: &tiberius::Row) -> Map<String, Value> where Self: Sized;
    #[doc(hidden)]
    fn row_to_struct(row: &tiberius::Row) -> Self where Self: Sized;

    /// Generate a query builder for the struct.
    fn query<'a>() -> QueryBuilder<'a, Self> where Self: Sized;

    /// Generate raw query instance for the struct that can be used to
    /// perform query with raw SQL string.
    /// ```
    /// # use ssql::prelude::*;
    /// # use chrono::NaiveDateTime;
    ///  #[derive(ORM)]
    ///  #[ssql(table)]
    ///  pub struct PersonRaw {
    ///     #[ssql(primary_key)]
    ///     id: i32,
    ///     email: String,
    ///     dt: Option<NaiveDateTime>
    ///  }
    ///
    ///  // make sure all fields in the struct are present in the query.
    ///  let query = PersonRaw::raw_query("SELECT id, email, dt FROM Person WHERE id = @p1", &[&1]);
    /// ```
    fn raw_query<'a>(sql: &str, params: &[&'a dyn ToSql]) -> QueryBuilder<'a, Self, RawQuery>
        where Self: Sized
    {
        let mut q = QueryBuilder {
            fields: Default::default(),
            filters: vec![],
            join: "".to_string(),
            tables: Default::default(),
            raw_sql: Some(sql.to_string()),
            relation_func: |_| "",
            query_params: vec![],
            query_idx_counter: 0,
            _marker: None,
            _mark2: Default::default(),
        };
        for p in params {
            q.query_params.push(*p);
        }
        q
    }

    /// Bulk insert, takes everything that can be turned into iterator that generate specific structs.
    /// ```no_run
    /// # use ssql::prelude::*;
    /// # #[derive(ORM)]
    /// # #[ssql(table = person)]
    /// # struct Person{
    /// #    id: i32,
    /// #    email: Option<String>,
    /// # }
    /// # async fn insert(mut conn: Client<Compat<TcpStream>>) {
    ///     // example1:
    ///     Person::insert_many(vec![Person{id: 1,email: Some("a@gmail.com".to_string())},
    ///                             Person{id: 2,email: Some("b@gmail.com".to_string())}], &mut conn).await;
    ///     // example2:
    ///     Person::insert_many((1..=3).zip(vec!["a@gmail.com", "b@gmail.com", "c@gmail.com"])
    ///         .map(|(idx, mail)| {
    ///                 Person {
    ///                     id: idx,
    ///                     email: Some(mail.to_string()),
    ///                 }
    ///         }), &mut conn).await
    /// # }
    /// ```
    async fn insert_many<I: IntoIterator<Item=Self> + Send>(iter: I, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<u64>
        where I::IntoIter: Send, Self: Sized;

    /// Insert one item, consume self.
    /// ```no_run
    /// # use ssql::prelude::*;
    ///  #[derive(ORM)]
    ///  #[ssql(table = person)]
    ///  struct Person{
    ///     id: i32,
    ///     email: Option<String>,
    ///  }
    /// # async fn insert(mut conn: Client<Compat<TcpStream>>) {
    ///  let person = Person{id: 1,email: Some("a@gmail.com".to_string())};
    ///  person.insert(&mut conn).await;
    /// # }
    /// ```
    /// SQL: `INSERT INTO person (id, email) VALUES ( 1, 'a@gmail.com')`
    async fn insert(self, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<()>;

    /// Insert one item while ignoring the primary key.
    /// Specified for those using `Identity` or `Auto-Increment` as primary key.
    /// If primary key is not set, this fn will perform as same as [`insert`]
    ///
    /// [`insert`]: trait.SsqlMarker.html#tymethod.insert
    ///
    /// ```no_run
    /// # use ssql::prelude::*;
    ///  #[derive(ORM)]
    ///  #[ssql(table = person)]
    ///  struct Person{
    ///     #[ssql(primary_key)]
    ///     id: i32,
    ///     email: Option<String>,
    ///  }
    /// # async fn insert(mut conn: Client<Compat<TcpStream>>) {
    ///  let person = Person{id: 1,email: Some("a@gmail.com".to_string())};
    ///  person.insert(&mut conn).await;
    /// # }
    /// ```
    /// SQL: `INSERT INTO person (email) VALUES ('a@gmail.com')`
    async fn insert_ignore_pk(self, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<()>;

    /// Delete one item based on primary key, consume self.
    /// Will panic if primary key is not set.
    /// ```no_run
    /// # use ssql::prelude::*;
    ///  #[derive(ORM)]
    ///  #[ssql(table = person)]
    ///  struct Person{
    ///     #[ssql(primary_key)]
    ///     id: i32,
    ///     email: Option<String>,
    ///  }
    ///  async fn _test(mut conn: Client<Compat<TcpStream>>) {
    ///     let person = Person{id: 1,email: Some("a@gmail.com".to_string())};
    ///     person.delete(&mut conn).await;
    ///  }
    /// ```
    /// SQL: `DELETE FROM person WHERE id = 1`
    async fn delete(self, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<()>;

    /// Update one item based on primary key, borrow self.
    /// Will panic if primary key is not set.
    /// ```no_run
    /// # use ssql::prelude::*;
    ///  #[derive(ORM)]
    ///  #[ssql(table = person)]
    ///  struct Person{
    ///      #[ssql(primary_key)]
    ///     id: i32,
    ///     email: Option<String>,
    ///  }
    ///  async fn _test(mut conn: Client<Compat<TcpStream>>) {
    ///     let person = Person{id: 1,email: Some("a@gmail.com".to_string())};
    ///     person.update(&mut conn).await;
    ///  }
    /// ```
    /// SQL: `UPDATE person SET email = 'a@gmail.com' WHERE id = 1`
    async fn update(&self, conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<()>;

    /// Generate a Column Expression that can be used in filtering and ordering.
    /// This method will failed if the given column name is no present in the struct.
    /// Thus it returns [`SsqlResult`]
    ///
    /// [`SsqlResult`]: type.SsqlResult.html
    fn col(field: &'static str) -> SsqlResult<ColExpr>
        where Self: Sized
    {
        match Self::fields().contains(&field) {
            true => {
                Ok(ColExpr { table: Self::table_name(), field })
            }
            false => {
                Err(format!("column {} not found in {}", field, Self::table_name()).into())
            }
        }
    }
}

/// Trait that automatically derive with `#[derive(ORM)]` when `polars` feature is enabled.
/// Supporting trait for [`get_dataframe`] method.
///
/// [`get_dataframe`]: struct.QueryBuilder.html#method.get_dataframe
#[cfg(feature = "polars")]
pub trait PolarsHelper {
    #[doc(hidden)]
    fn dataframe(vec: Vec<Self>) -> PolarsResult<DataFrame>
        where Self: Sized;
}

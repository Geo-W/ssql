use async_trait::async_trait;
#[cfg(feature = "polars")]
use polars::prelude::*;
#[cfg(feature = "polars")]
use tiberius::QueryStream;
use serde_json::{Map, Value};
use tiberius::{Client, ToSql};
use tokio::net::TcpStream;
use tokio_util::compat::Compat;

use crate::{ColExpr, QueryBuilderI, SsqlResult};
use crate::structs::query_core::QueryCore;
use crate::structs::raw_query_builder::RawQueryBuilder;

/// a trait automatically derived via `#[derive(ORM)]` macro, all these methods are available.
#[async_trait]
pub trait SsqlMarker {
    #[doc(hidden)]
    fn table_name() -> &'static str
    where
        Self: Sized;
    #[doc(hidden)]
    fn fields() -> Vec<&'static str>
    where
        Self: Sized;
    #[doc(hidden)]
    fn row_to_json(row: &tiberius::Row) -> Map<String, Value>
    where
        Self: Sized;
    #[doc(hidden)]
    fn row_to_struct(row: &tiberius::Row) -> Self
    where
        Self: Sized;

    #[doc(hidden)]
    #[cfg(feature = "polars")]
    async fn dataframe<'a>(stream: QueryStream<'a>) -> SsqlResult<DataFrame>
    where
        Self: Sized;

    /// Generate a query builder for the struct.
    fn query<'a>() -> QueryBuilderI<'a, Self>
    where
        Self: Sized;

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
    fn raw_query<'a>(sql: &str, params: &[&'a dyn ToSql]) -> RawQueryBuilder<'a, Self>
    where
        Self: Sized,
    {
        let mut q= QueryCore::default();
        q.raw_sql = Some(sql.to_string());
        for p in params {
            q.query_params.push(*p);
        }
        RawQueryBuilder{ core: q, t: Default::default() }
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
    /// # async fn insert(mut conn: Client<Compat<TcpStream>>) -> SsqlResult<u64> {
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
    async fn insert_many<I: IntoIterator<Item = Self> + Send>(
        iter: I,
        conn: &mut Client<Compat<TcpStream>>,
    ) -> SsqlResult<u64>
    where
        I::IntoIter: Send,
        Self: Sized;

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

    #[doc(hidden)]
    fn relationship(input: &str) -> &'static str where Self: Sized;

    #[doc(hidden)]
    fn primary_key(&self) -> (&'static str, &dyn ToSql);
    /// Generate a Column Expression that can be used in filtering and ordering.
    /// This method will failed if the given column name is no present in the struct.
    /// Thus it returns [`SsqlResult`]
    ///
    /// [`SsqlResult`]: type.SsqlResult.html
    fn col(field: &'static str) -> SsqlResult<ColExpr>
    where
        Self: Sized,
    {
        match Self::fields().contains(&field) {
            true => Ok(ColExpr {
                table: Self::table_name(),
                field,
            }),
            false => Err(format!("column {} not found in {}", field, Self::table_name()).into()),
        }
    }
}

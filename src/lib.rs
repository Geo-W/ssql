//! An easy-to-use asynchronous ms sql server ORM.
//!
//! # Way of Getting Data.
//! Query is built using [`Table::query`] method, it returns [`QueryBuilder`] object.
//! Through this object u can join other tables, apply filters, make query or make bulk inserts.
//! ```
//! use ssql::prelude::*;
//! use serde::{Deserialize, Serialize};
//! use chrono::NaiveDateTime;
//!
//! #[derive(ORM, Debug, Default, Serialize, Deserialize)]
//! #[ssql(table = person, schema = SCHEMA1)] // other schema
//! struct Person {
//!     #[ssql(primary_key)]
//!     id: i32,
//!     email: Option<String>, // wrap nullable column in option
//! }
//!
//! #[derive(ORM, Debug, Default, Serialize, Deserialize)]
//! #[ssql(table = posts)] // default schema
//! struct Posts {
//!     id: i32,
//!     post: String,
//!     #[ssql(foreign_key = "SCHEMA1.Person.id")] // if it belongs to default schema, just write TABLE.COLUMN
//!     person_id: i32,
//! }
//!
//! async fn _get<'a>(client: &'a mut tiberius::Client<Compat<TcpStream>>) -> SsqlResult<()> {
//!         let mut query = Person::query()
//!         .join::<Posts>();
//!
//!     // return a vector of struct
//!     let vec1 = query.get_struct::<Posts>(client).await?;
//!     let (vec1, vec2) = query.get_struct_2::<Person, Posts>(client).await?;
//!
//!     // return a vector of serde_json::Value;
//!     let vec1 = query.get_serialized::<Person>(client).await?;
//!
//!     // with polars feature enabled, return DataFrame;
//!     let (df1, df2) = query.get_dataframe_2::<Person, Posts>(client).await?;
//!
//!
//!     Ok(())
//! }
//!
//! ```
//!
//! # Filters
//! Filters can be applied to query builder via provided [`filter`] method.
//! Filters can be chained.
//! For all filter expression please refer to [`ColExpr`].
//! ```no_run
//! # use ssql::prelude::*;
//! # use serde::{Deserialize, Serialize};
//! # #[derive(ORM, Debug, Default, Serialize, Deserialize)]
//! # #[ssql(table = Person, schema = SCHEMA1)] // default schema
//! # struct Person {
//! #     #[ssql(primary_key)]
//! #     id: i32,
//! #     email: Option<String>, // wrap nullable column in option
//! # }
//! # let query = Person::query();
//! let query = query.filter(
//!     Person::col("email")?.eq(&"abc@gmail.com")
//! ).filter(
//!     Person::col("id")?.gt(&3)
//! );
//! ```
//!
//! # Manipulating Data
//! Data can be [`insert`],[`delete`],[`update`],[`insert_ignore_pk`] for any instance that `#[derive(ORM)]` and set `#[ssql(primary_key)]`.
//! Or calling `bulk insert` with [`Struct::insert_many(&mut conn)`] method.
//!
//! [`insert`]: trait.SsqlMarker.html#tymethod.insert
//! [`delete`]: trait.SsqlMarker.html#tymethod.delete
//! [`update`]: trait.SsqlMarker.html#tymethod.update
//! [`insert_ignore_pk`]: trait.SsqlMarker.html#tymethod.insert_ignore_pk
//! [`Struct::insert_many(&mut conn)`]: trait.SsqlMarker.html#tymethod.insert_many
//! ```
//! # use ssql::prelude::*;
//! # use serde::{Deserialize, Serialize};
//! # use chrono::NaiveDateTime;
//! # #[derive(ORM, Debug, Default, Serialize, Deserialize)]
//! # #[ssql(table = person, schema = SCHEMA1)]
//! # struct Person {
//! #     #[ssql(primary_key)]
//! #     id: i32,
//! #     email: Option<String>,
//! # }
//! async fn _test<'a>(client: &'a mut tiberius::Client<Compat<TcpStream>>) -> SsqlResult<()> {
//!     let new_p = Person {
//!         id: 2,
//!         email: Some("a@a.com".to_string()),
//!     };
//!
//!     //insert with data in this instance.
//!     new_p.insert(client).await?;
//!
//!     //insert with data in this instance ignoring the primary key.
//!     new_p.insert_ignore_pk(client).await?;
//!
//!     // delete it based on its primary key mark.
//!     // like here i mark id with #[ssql(primary_key)]
//!     new_p.delete(client).await?;
//!
//!     // update it based on its primary key mark.
//!     new_p.update(client).await?;
//!
//!
//!     // insert many accepts anything that can turn into iterator and return specific type, here is <Person>
//!     let vec = vec![new_p.clone(), new_p.clone()];
//!     Person::insert_many(vec, client).await?;
//!
//!     let it = vec![1, 2, 3].into_iter().zip(
//!         vec!["a", "b", "c"].into_iter()
//!     ).map(|(id, email)| Person {
//!         id,
//!         email: Some(email.to_string()),
//!     });
//!     Person::insert_many(it, client).await?;
//!     Ok(())
//! }
//! ```
//!
//! # Raw Sql Query
//! Using [`raw_query`] method to construct a raw sql query.
//! Field name are reflecting as column name in sql query result.
//! ```
//! use ssql::prelude::*;
//! # use chrono::NaiveDateTime;
//!  // structs reflecting complex raw query
//!  // leave the table attribute empty
//!  #[derive(ORM)]
//!  #[ssql(table)]
//!  pub struct PersonRaw {
//!     #[ssql(primary_key)]
//!     id: i32,
//!     email: String,
//!     dt: Option<NaiveDateTime>
//!  }
//! async fn _get<'a>(client: &'a mut tiberius::Client<Compat<TcpStream>>) -> SsqlResult<()> {
//!
//! let query = PersonRaw::raw_query("SELECT id, email, dt FROM Person where id = @p1", &[&1]);
//!  let data = query.get_struct::<PersonRaw>(client).await;
//!  Ok(())
//! }
//! ```
//! [`filter`]: struct.QueryBuilder.html#method.filter
//! [`Table::query`]: trait.SsqlMarker.html#tymethod.query
//! [`ColExpr`]: structs.filter.ColExpr.html
//! [`QueryBuilder`]: struct.QueryBuilder.html
//! [`raw_query`]: trait.SsqlMarker.html#method.raw_query
#![warn(missing_docs)]
#[macro_use]
pub(crate) mod macros;
mod structs;

mod error;
/// All necessary imports for using this crate.
pub mod prelude;

/// Utility functions.
pub mod utils;

pub use error::custom_error::SsqlError;
pub use error::custom_error::SsqlResult;

pub use structs::filter::ColExpr;
pub use structs::filter::FilterExpr;
pub use structs::querybuilder::QueryCore;
pub use structs::querybuilder::SsqlMarker;
pub use structs::stream::RowStream;

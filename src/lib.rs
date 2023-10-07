//! An easy-to-use asynchronous ms sql server ORM.
//!
//! # Way of Getting Data.
//! Query is built using [`Table::query`] method.
//! ```
//! use ssql::prelude::*;
//! use serde::{Deserialize, Serialize};
//! use chrono::NaiveDateTime;
//!
//! #[derive(ORM, Debug, Default, Serialize, Deserialize)]
//! #[ssql(table = person, schema = SCHEMA1)] // default schema
//! struct Person {
//!     #[ssql(primary_key)]
//!     id: i32,
//!     email: Option<String>, // wrap nullable column in option
//! }
//!
//! #[derive(ORM, Debug, Default, Serialize, Deserialize)]
//! #[ssql(table = posts)] // other schema
//! struct Posts {
//!     id: i32,
//!     post: String,
//!     #[ssql(foreign_key = "SCHEMA1.Person.id")] // if it belongs to default schema, just write TABLE.COLUMN
//!     person_id: i32,
//! }
//!
//! async fn get<'a>(client: &'a mut tiberius::Client<Compat<TcpStream>>) -> SsqlResult<()> {
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
//!     let new_p = Person {
//!         id: 2,
//!         email: Some("a@a.com".to_string()),
//!     };
//!
//!     //insert with data in this instance.
//!     new_p.insert(client);
//!
//!     // delete it based on its primary key mark.
//!     // like here i mark id with #[ssql(primary_key)]
//!     new_p.delete(client);
//!
//!     // update it based on its primary key mark.
//!     new_p.update(client);
//!
//!
//!     // insert many accepts anything that can turn into iterator and return specific type, here is <Person>
//!     let vec = vec![new_p.clone(), new_p.clone()];
//!     Person::insert_many(vec, client);
//!
//!     let it = vec![1, 2, 3].into_iter().zip(
//!         vec!["a", "b", "c"].into_iter()
//!     ).map(|(id, email)| Person {
//!         id,
//!         email: Some(email.to_string()),
//!     });
//!     Person::insert_many(it, client);
//!
//!     // structs reflecting complex raw query
//!     // leave the table attribute empty
//!     #[derive(ORM, Debug, Default, Serialize, Deserialize)]
//!     #[ssql(table)]
//!     pub struct PersonRaw {
//!         #[ssql(primary_key)]
//!         pub(crate) id: i32,
//!         pub(crate) email: String,
//!         dt: Option<NaiveDateTime>
//!     }
//!
//!     let query = PersonRaw::query()
//!         .raw("SELECT * FROM Person where id = @p1", &[&1]);
//!
//!     let data = query.get_struct::<PersonRaw>(client).await;
//! }
//!
//! ```
//!
//! # Filters
//! Filters can be applied to query builder via provided [`filter`] method.
//! Filters can be chained.
//! For all filter expression please refer to [`crate::structs::filter::ColExpr`]
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
//! [`filter`]: struct.QueryBuilder.html#method.filter
//! [`Table::query`]: trait.SsqlMarker.html#tymethod.query
//! [`Col`]: structs.filter.ColExpr.html

mod structs;
pub mod prelude;
pub mod error;
pub mod utils;
mod macros;

pub use structs::filter::ColExpr;
pub use structs::filter::FilterExpr;
pub use structs::querybuilder::QueryBuilder;
pub use structs::querybuilder::SsqlMarker;
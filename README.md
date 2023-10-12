# An easy-to use basic mssql server ORM based on tiberius.  

This crate is still under construction, apis may subject to change.   
For full documentation pls visit [doc.rs](https://docs.rs/ssql/*/ssql/).
### Quick Glance:
> When defining structs, make sure keep the field sequence consistent with the sequence in database as bulk insert(insert_many) depends on it. 
```rust
use ssql::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(ORM, Debug, Default, Serialize, Deserialize)]
#[ssql(table = Person, schema = SCHEMA1)] // other schema
struct Person {
    #[ssql(primary_key)]
    id: i32,
    email: Option<String>, // wrap nullable column in option
}

#[derive(ORM, Debug, Default, Serialize, Deserialize)]
#[ssql(table = Posts)] // default schema
struct Posts {
    id: i32,
    post: String,
    #[ssql(foreign_key = "SCHEMA1.Person.id")] // if it belongs to default schema, just write TABLE.COLUMN
    person_id: i32,
}

async fn get<'a>(client: &'a mut tiberius::Client<Compat<TcpStream>>) -> SsqlResult<()> {
    let mut query = Person::query()
        .join::<Posts>();

    // return a vector of struct
    let vec1 = query.get_struct::<Posts>(client).await?;
    
    let (vec1, vec2) = query.get_struct_2::<Person, Posts>(client).await?;

    // return a vector of serde_json::Value;
    let vec1 = query.get_serialized::<Person>(client).await?;

    // with polars feature enabled, return DataFrame;
    let (df1, df2) = query.get_dataframe_2::<Person, Posts>(client).await?;
}
```


### TODO:
- [ ] handle multiple relationships
- [x] build filter pattern
- [x] support raw sql string query
- [x] handle non-manual input key like auto-generated id
- [ ] handle `GROUP BY` aggregation
- [ ] support filter with decorated col like `WHERE YEAR(datetime_col) = ?`
# An easy-to use basic mssql server ORM based on tiberius.  

This crate is still under construction, apis may subject to change.   
For now only the basic function is accomplished, check it out below.
### Usage:

```rust
use rssql::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(ORM, Debug, Default, Serialize, Deserialize)]
#[rssql(table = Person, schema = SCHEMA1)] // default schema
struct Person {
    #[rssql(primary_key)]
    id: i32,
    email: Option<String>, // wrap nullable column in option
}

#[derive(ORM, Debug, Default, Serialize, Deserialize)]
#[rssql(table = Posts)] // other schema
struct Posts {
    id: i32,
    post: String,
    #[rssql(foreign_key = "SCHEMA1.Person.id")] // if it belongs to default schema, just write TABLE.COLUMN
    person_id: i32,
}

async fn get<'a>(client: &'a mut tiberius::Client<Compat<TcpStream>>) -> RssqlResult<()> {
    let mut query = Person::query()
        .join::<Posts>();

    // return a vector of struct
    let vec1 = query.get_struct::<Posts>(&mut client).await?;
    let (vec1, vec2) = query.get_struct_2::<Person, Posts>(&mut client).await?;

    // return a vector of serde_json::Value;
    let vec1 = query.get_serialized::<Person>(&mut client).await?;

    // with polars feature enabled, return DataFrame;
    let (df1, df2) = query.get_dataframe_2::<Person, Posts>(&mut client).await?;

    let new_p = Person {
        id: 2,
        email: Some("a@a.com".to_string()),
    };

    //insert with data in this instance.
    new_p.insert(&mut client);

    // delete it based on its primary key mark.
    // like here i mark id with #[rssql(primary_key)]
    new_p.delete(&mut client);

    // update it based on its primary key mark.
    new_p.update(&mut client);


    // insert many accepts anything that can turn into iterator and return specific type, here is <Person>
    let vec = vec![new_p.clone(), new_p.clone()];
    Person::insert_many(vec, &mut client);

    let it = vec![1, 2, 3].into_iter().zip(
        vec!["a", "b", "c"].into_iter()
    ).map(|(id, email)| Person {
        id,
        email: email.to_string(),
    });
    Person::insert_many(it, &mut client);
}


```


### TODO:
> 1. handling multiple relationships
> 2. build filter pattern
> 3. support raw sql string query
> 4. handle non-manual input key like auto-generated id
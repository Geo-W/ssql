#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use tiberius::Client;
    use tokio::net::TcpStream;
    use tokio_util::compat::Compat;

    use ssql::prelude::*;

    #[tokio::test]
    async fn test() {
        // let mut client = get_client().await;
        // let mut query = Customerlist::query();
        // query.find_all(&mut client).await.unwrap();
        // .join::<Test>();
    }
}


use std::alloc;
use std::vec::IntoIter;
use ssql::prelude::*;

pub struct Person {
    pub(crate) id: i32,
    pub(crate) Email: String,
}

impl Person {
    fn primary_key(&self) -> (&'static str, &dyn ToSql) {
        ("id", &self.id)
    }
}

#[async_trait]
impl SsqlMarker for Person {
    fn table_name() -> &'static str {
        "Person"
    }
    fn fields() -> Vec<&'static str> {
        vec![]
    }
    fn row_to_json(row: &Row) -> Map<String, Value> {
        let mut map = Map::new();
        map.insert("id".to_string(), row.get::<i32, &str>("Person.id").into());
        map.insert(
            "Email".to_string(),
            row.get::<&str, &str>("Person.Email").into(),
        );
        map
    }
    fn row_to_struct(row: &Row) -> Self {
        Self {
            id: row.get::<i32, &str>("Person.id").unwrap(),
            Email: row.get::<&str, &str>("Person.Email").unwrap().to_string(),
        }
    }
    fn query<'a>() -> QueryBuilder<'a, Person> {
        QueryBuilder::<
            Person,
        >::new(("Person", Person::fields()), Person::relationship)
    }
    // #[allow(
    // clippy::async_yields_async,
    // clippy::let_unit_value,
    // clippy::no_effect_underscore_binding,
    // clippy::shadow_same,
    // clippy::type_complexity,
    // clippy::type_repetition_in_bounds,
    // clippy::used_underscore_binding
    // )]
    // fn insert_many2<'life0, 'async_trait>(
    //     iter: impl 'async_trait + IntoIterator<Item=Person> + Send,
    //     conn: &'life0 mut Client<Compat<TcpStream>>,
    // ) -> ::core::pin::Pin<
    //     Box<
    //         dyn ::core::future::Future<
    //             Output=SsqlResult<u64>,
    //         > + ::core::marker::Send + 'async_trait,
    //     >,
    // >
    //     where
    //         'life0: 'async_trait,
    // {
    //     Box::pin(async move {
    //         if let ::core::option::Option::Some(__ret)
    //             = ::core::option::Option::None::<SsqlResult<u64>> {
    //             return __ret;
    //         }
    //         let iter = iter;
    //         let __ret: SsqlResult<u64> = {
    //             let mut req = conn.bulk_insert("Person").await?;
    //             for item in iter {
    //                 let mut row = TokenRow::new();
    //                 row.push(item.id.into_sql());
    //                 row.push(item.Email.into_sql());
    //                 req.send(row).await?;
    //             }
    //             let res = req.finalize().await?;
    //             Ok(res.total())
    //         };
    //         #[allow(unreachable_code)] __ret
    //     })
    // }

    async fn insert_many(iter:impl IntoIterator<Item=Person, IntoIter=impl Iterator<Item=Person> + Send> + Send,
                         conn: &mut Client<Compat<TcpStream>>) -> SsqlResult<u64>
    // where <Iterator<Item=Person> as IntoIterator>::IntoIter: Send
    //     <dyn IntoIterator<Item=Person> + Send  as IntoIterator>::IntoIter: Send
    {
        let a = iter.into_iter();
        let mut req = conn.bulk_insert("Person").await?;
        // for item in iter {
        //     let mut row = TokenRow::new();
        //     row.push(item.id.into_sql());
        //     row.push(item.Email.into_sql());
        //     // req.send(row).await?;
        // }
        let res = req.finalize().await?;
        Ok(res.total())
    }


    #[allow(
    clippy::async_yields_async,
    clippy::let_unit_value,
    clippy::no_effect_underscore_binding,
    clippy::shadow_same,
    clippy::type_complexity,
    clippy::type_repetition_in_bounds,
    clippy::used_underscore_binding
    )]
    fn insert<'life0, 'async_trait>(
        self,
        conn: &'life0 mut Client<Compat<TcpStream>>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output=SsqlResult<()>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
        where
            'life0: 'async_trait,
            Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret)
                = ::core::option::Option::None::<SsqlResult<()>> {
                return __ret;
            }
            let __self = self;
            let __ret: SsqlResult<()> = {
                let sql = "";
                conn.execute(sql, &[&__self.id, &__self.Email]).await?;
                Ok(())
            };
            #[allow(unreachable_code)] __ret
        })
    }
    #[allow(
    clippy::async_yields_async,
    clippy::let_unit_value,
    clippy::no_effect_underscore_binding,
    clippy::shadow_same,
    clippy::type_complexity,
    clippy::type_repetition_in_bounds,
    clippy::used_underscore_binding
    )]
    fn insert_ignore_pk<'life0, 'async_trait>(
        self,
        conn: &'life0 mut Client<Compat<TcpStream>>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output=SsqlResult<()>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
        where
            'life0: 'async_trait,
            Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret)
                = ::core::option::Option::None::<SsqlResult<()>> {
                return __ret;
            }
            let __self = self;
            let __ret: SsqlResult<()> = {
                let sql = "";
                conn.execute(sql, &[&__self.Email]).await?;
                Ok(())
            };
            #[allow(unreachable_code)] __ret
        })
    }
    #[allow(
    clippy::async_yields_async,
    clippy::let_unit_value,
    clippy::no_effect_underscore_binding,
    clippy::shadow_same,
    clippy::type_complexity,
    clippy::type_repetition_in_bounds,
    clippy::used_underscore_binding
    )]
    fn delete<'life0, 'async_trait>(
        self,
        conn: &'life0 mut Client<Compat<TcpStream>>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output=SsqlResult<()>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
        where
            'life0: 'async_trait,
            Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret)
                = ::core::option::Option::None::<SsqlResult<()>> {
                return __ret;
            }
            let __self = self;
            let __ret: SsqlResult<()> = {
                let (pk, dt) = __self.primary_key();
                conn.execute(
                    "",
                    &[dt],
                )
                    .await?;
                Ok(())
            };
            #[allow(unreachable_code)] __ret
        })
    }
    #[allow(
    clippy::async_yields_async,
    clippy::let_unit_value,
    clippy::no_effect_underscore_binding,
    clippy::shadow_same,
    clippy::type_complexity,
    clippy::type_repetition_in_bounds,
    clippy::used_underscore_binding
    )]
    fn update<'life0, 'life1, 'async_trait>(
        &'life0 self,
        conn: &'life1 mut Client<Compat<TcpStream>>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output=SsqlResult<()>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
        where
            'life0: 'async_trait,
            'life1: 'async_trait,
            Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret)
                = ::core::option::Option::None::<SsqlResult<()>> {
                return __ret;
            }
            let __self = self;
            let __ret: SsqlResult<()> = {
                let (pk, dt) = __self.primary_key();
                let sql = "";
                conn.execute(sql, &[&__self.id, &__self.Email, dt]).await?;
                Ok(())
            };
            #[allow(unreachable_code)] __ret
        })
    }
}


#[automatically_derived]
impl ::core::default::Default for Person {
    #[inline]
    fn default() -> Person {
        Person {
            id: ::core::default::Default::default(),
            Email: ::core::default::Default::default(),
        }
    }
}

impl Person {
    fn relationship(input: &str) -> &'static str {
        "input"
    }
    fn column_type(input: &str) -> &'static str {
        match input {
            "id" => "i32",
            "Email" => "String",
            _ => {
                ""
            }
        }
    }
}
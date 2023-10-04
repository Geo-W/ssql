#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use tiberius::Client;
    use tokio::net::TcpStream;
    use tokio_util::compat::Compat;

    use rssql::prelude::*;

    #[tokio::test]
    async fn test() {
        // let mut client = get_client().await;
        // let mut query = Customerlist::query();
        // query.find_all(&mut client).await.unwrap();
        // .join::<Test>();
    }


    pub struct Person {
        pub(crate) id: i32,
        pub(crate) Email: String,
    }

    impl Person {
        fn primary_key(&self) -> (&'static str, ColumnData) {
            ("id", self.id.to_sql())
        }
    }

    impl RssqlMarker for Person {
        fn table_name() -> &'static str {
            "Person"
        }
        fn fields() -> Vec<&'static str> {
            vec!["aSdf"]
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
        fn row_to_self(row: &Row) -> Self {
            Self {
                id: row.get::<i32, &str>("Person.id").unwrap(),
                Email: row.get::<&str, &str>("Person.Email").unwrap().to_string(),
            }
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
        fn insert_many<'life0, 'async_trait>(
            iter: impl 'async_trait + IntoIterator<Item=Person>,
            conn: &'life0 mut Client<Compat<TcpStream>>,
        ) -> ::core::pin::Pin<
            Box<dyn ::core::future::Future<Output=RssqlResult<u64>> + 'async_trait>,
        >
            where
                'life0: 'async_trait,
        {
            Box::pin(async move {
                if let ::core::option::Option::Some(__ret)
                    = ::core::option::Option::None::<RssqlResult<u64>> {
                    return __ret;
                }
                let iter = iter;
                let __ret: RssqlResult<u64> = {
                    let mut req = conn.bulk_insert("Person").await?;
                    for item in iter {
                        let mut row = TokenRow::new();
                        row.push(item.id.into_sql());
                        row.push(item.Email.into_sql());
                        req.send(row).await?;
                    }
                    let res = req.finalize().await?;
                    Ok(res.total())
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
        fn insert_one<'life0, 'async_trait>(
            self,
            conn: &'life0 mut Client<Compat<TcpStream>>,
        ) -> ::core::pin::Pin<
            Box<dyn ::core::future::Future<Output=RssqlResult<()>> + 'async_trait>,
        >
            where
                'life0: 'async_trait,
                Self: 'async_trait,
        {
            Box::pin(async move {
                if let ::core::option::Option::Some(__ret)
                    = ::core::option::Option::None::<RssqlResult<()>> {
                    return __ret;
                }
                let __self = self;
                let __ret: RssqlResult<()> = {
                    let sql = {
                        let res =
                            format!(
                                "INSERT INTO {0} ({1}) values({2})", "Person", "id,Email",
                                "@p1,@p2"
                            )
                        ;
                        res
                    };
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
        fn delete<'life0, 'async_trait>(
            self,
            conn: &'life0 mut Client<Compat<TcpStream>>,
        ) -> ::core::pin::Pin<
            Box<dyn ::core::future::Future<Output=RssqlResult<()>> + 'async_trait>,
        >
            where
                'life0: 'async_trait,
                Self: 'async_trait,
        {
            Box::pin(async move {
                if let ::core::option::Option::Some(__ret)
                    = ::core::option::Option::None::<RssqlResult<()>> {
                    return __ret;
                }
                let __self = self;
                let __ret: RssqlResult<()> = {
                    let (pk, dt) = __self.primary_key();
                    QueryBuilder::<Person>::delete(&dt, "Person", pk, conn).await?;
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
            Box<dyn ::core::future::Future<Output=RssqlResult<()>> + 'async_trait>,
        >
            where
                'life0: 'async_trait,
                'life1: 'async_trait,
                Self: 'async_trait,
        {
            Box::pin(async move {
                if let ::core::option::Option::Some(__ret)
                    = ::core::option::Option::None::<RssqlResult<()>> {
                    return __ret;
                }
                let __self = self;
                let __ret: RssqlResult<()> = {
                    let (pk, dt) = __self.primary_key();
                    let sql = {
                        let res =
                            format!(
                                "UPDATE {0} SET {1} WHERE {2} {3}", "Person",
                                " id = @p1, Email = @p2", pk,
                                QueryBuilder::<Person>::process_pk_condition(&dt)
                            )
                        ;
                        res
                    };
                    conn.execute(sql, &[&__self.id, &__self.Email]).await?;
                    Ok(())
                };
                #[allow(unreachable_code)] __ret
            })
        }
    }

    impl Person {
        fn relationship(input: &str) -> &'static str {
            match input {
                _ => {
                    unimplemented!()
                }
            }
        }
        fn column_type(input: &str) -> &'static str {
            match input {
                "id" => "i32",
                "Email" => "String",
                _ => {
                    unimplemented!()
                }
            }
        }
        pub fn query() -> QueryBuilder<Person> {
            QueryBuilder::<Person, Person>::new(
                "Person",
                ("Person", Person::fields()),
                Person::relationship,
                Box::new(Person::row_to_json),
            )
        }
    }
}

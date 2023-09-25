#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveTime, NaiveDateTime};
    use serde::{Deserialize, Serialize};
    use tiberius::Client;
    use tokio::net::TcpStream;
    use tokio_util::compat::Compat;

    use ssql::prelude::*;


    #[tokio::test]
    async fn test() {
        let mut client = get_client().await;
        let mut query = Customerlist::query();
        let a = query.get_struct::<Customerlist>(&mut client).await;
        dbg!(a.unwrap());
        let b = query.get_serialized::<Customerlist>(&mut client).await;
        dbg!(b.unwrap());
        let mut c = query.left_join::<SlowMoving>();
        c.get_struct_2::<Customerlist, SlowMoving>(&mut client).await.unwrap();
    }

    #[tokio::test]
    async fn filter() -> SsqlResult<()> {
        let mut client = get_client().await;
        let mut query = Customerlist::query()
            .filter(
                Customerlist::col("ship_to_id")?.eq(&"1239706")
            )?;
            // .filter(
            //     Customerlist::col("volume")?.eq(&666)
            // )?;
        let a = query.get_struct::<Customerlist>(&mut client).await?;
        dbg!(a);
        Ok(())
    }

    #[tokio::test]
    async fn insert_many() {
        let mut conn = get_client().await;
        let it = vec![Person { id: 5, Email: "a".to_string() }, Person { id: 6, Email: "a".to_string() }].into_iter();
        let _a = Person::insert_many(it, &mut conn).await;
        let it = vec![Person { id: 5, Email: "a".to_string() }, Person { id: 6, Email: "a".to_string() }];
        let a = Person::insert_many(it, &mut conn).await;
        dbg!(&a);
    }

    #[tokio::test]
    async fn insert_one() {
        let mut conn = get_client().await;
        let item = Person {
            id: 1,
            Email: "".to_string(),
        };
        let ret = item.insert(&mut conn).await;
        assert_eq!(ret.is_ok(), true);
    }

    #[tokio::test]
    async fn delete() {
        let p = Person {
            id: 1,
            Email: "".to_string(),
        };
        let mut conn = get_client().await;
        assert_eq!(p.delete(&mut conn).await.is_ok(), true);
    }

    #[tokio::test]
    async fn update() {
        let p = Person {
            id: 99,
            Email: "".to_string(),
        };
        let mut conn = get_client().await;
        assert_eq!(p.update(&mut conn).await.is_ok(), true);
    }


    #[tokio::test]
    async fn raw_query_and_chrono() {
        use chrono;
        let mut conn = get_client().await;
        let now = chrono::NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2022, 5, 5).unwrap(),
            NaiveTime::from_hms_micro_opt(1, 1, 1, 0).unwrap(),
        );
        let mut m = PersonRaw::query().raw("SELECT * FROM Person where id = @p1", &[&"asdf"]);
        let m = m.get_struct::<PersonRaw>(&mut conn).await;
        assert_eq!(m.is_ok(), true);
    }

    pub async fn get_client() -> Client<Compat<TcpStream>> {
        ssql::utils::get_client("username", "password", "host", "database").await
    }

    #[derive(ORM, Debug, Default, Serialize, Deserialize)]
    #[ssql(table = CUSTOMER_LIST, schema = MASTER_DATA)]
    pub struct Customerlist {
        pub(crate) ship_to_id: Option<String>,
        #[ssql(foreign_key = "DALI_DATA.SLOW_MOVING.stock_in_day")]
        pub(crate) ship_to: Option<String>,
        pub(crate) volume: Option<i32>,
        pub(crate) container: Option<String>,
    }

    #[derive(ORM, Debug, Default)]
    #[ssql(table = SLOW_MOVING, schema = DALI_DATA)]
    pub struct SlowMoving {
        pub(crate) stock_in_day: Option<String>,
        pub(crate) total_value: Option<f64>,
        pub(crate) Week: Option<i64>,
        // pub(crate) Generated_Time: Option<NaiveDateTime>,
    }

    #[derive(ORM, Debug, Default)]
    #[ssql(table = Person)]
    pub struct Person {
        #[ssql(primary_key)]
        pub(crate) id: i32,
        pub(crate) Email: String,
    }

    #[derive(ORM, Debug, Default, Serialize, Deserialize)]
    #[ssql(table)]
    pub struct PersonRaw {
        #[ssql(primary_key)]
        pub(crate) id: i32,
        pub(crate) Email: String,
        dt: Option<NaiveDateTime>,
    }

    #[derive(ORM, Debug, Default)]
    #[ssql(table = FORECAST)]
    pub struct Fcst {
        pub(crate) Customer: Option<String>,
        pub(crate) Material: Option<String>,
        Dv: Option<f64>,
        Route: Option<String>,
        TransitTime: Option<String>,
        Plant: Option<String>,
    }
}


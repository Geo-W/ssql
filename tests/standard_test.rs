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
        let mut client = get_client().await;
        let mut query = Customerlist::query();
        let a = query.get_struct::<Customerlist>(&mut client).await;
        dbg!(a);
        // let b = query.get_serialized::<Customerlist>(&mut client).await;
        // dbg!(b.unwrap());
        // .join::<Test>();
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
            id: 99,
            Email: "".to_string(),
        };
        let ret = item.insert_one(&mut conn).await;
        assert_eq!(ret.is_ok(), true);
    }

    #[tokio::test]
    async fn delete() {
        let p = Person {
            id: 0,
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
        assert_eq!(p.delete(&mut conn).await.is_ok(), true);
    }


    pub async fn get_client() -> Client<Compat<TcpStream>> {
        rssql::utils::get_client("username", "password", "host", "database").await
    }

    #[derive(ORM, Debug, Default, Serialize, Deserialize)]
    #[rusql(table = CUSTOMER_LIST, schema = MASTER_DATA)]
    pub struct Customerlist {
        pub(crate) ship_to_id: Option<String>,
        #[rusql(foreign_key = "SLOW_MOVING.stock_in_day")]
        pub(crate) ship_to: Option<String>,
        pub(crate) volume: Option<i32>,
        pub(crate) container: Option<String>,
    }

    #[derive(ORM, Debug, Default)]
    #[rusql(table = SLOW_MOVING)]
    pub struct SlowMoving {
        pub(crate) stock_in_day: Option<String>,
        pub(crate) total_value: Option<f64>,
        pub(crate) Week: Option<i64>,
        // pub(crate) Generated_Time: Option<NaiveDateTime>,
    }

    #[derive(ORM, Debug, Default)]
    #[rusql(table = Person)]
    pub struct Person {
        #[rusql(primary_key)]
        pub(crate) id: i32,
        pub(crate) Email: String,
    }


    #[derive(ORM, Debug, Default)]
    #[rusql(table = FORECAST)]
    pub struct Fcst {
        pub(crate) Customer: Option<String>,
        pub(crate) Material: Option<String>,
        Dv: Option<f64>,
        Route: Option<String>,
        TransitTime: Option<String>,
        Plant: Option<String>,
    }


    // #[derive(ORM, Debug, Default)]
    // #[rusql(table = SA)]
    // pub struct Sa {
    //     sa_qty: i64,
    //     material: String,
    //     description: String,
    //     eta: String,
    //     vendor: String,
    //     vendor_id: String,
    //     planner: String,
    //     Generated_Time: NaiveDateTime,
    // }
}


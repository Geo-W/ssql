#[allow(non_snake_case)]
#[cfg(feature = "polars")]
#[cfg(test)]
mod tests {
    use tokio::net::TcpStream;
    use tokio_util::compat::Compat;

    use rssql::prelude::*;
    use tiberius::Client;

    #[cfg(feature = "polars")]
    #[tokio::test]
    async fn data_frame() {
        let mut client = get_client().await;
        let df = Customerlist::query()
            .get_dataframe::<Customerlist>(&mut client).await;
        println!("{:?}", df.unwrap());
        assert_eq!(1, 1);
    }

    #[tokio::test]
    async fn data_frame2() {
        let mut client = get_client().await;
        let (df1, df2) = Customerlist::query()
            .join::<Test>()
            .get_dataframe_2::<Customerlist, Test>(&mut client).await.unwrap();
        println!("{:?}", df1);
        println!("{:?}", df2);
        assert_eq!(1, 1);
    }

    #[tokio::test]
    async fn it_works2() {
        let mut client = get_client().await;
        let now = std::time::Instant::now();
        let mut query = Fcst::query();
        query.find_all(&mut client).await.unwrap();
        dbg!(now.elapsed());

        let now = std::time::Instant::now();
        let query = Fcst::query().get_dataframe::<Fcst>(&mut client).await.unwrap();
        dbg!(&query);
        dbg!(now.elapsed());
    }

    #[test]
    fn aaa() {
        let df = df!(
            "asdf" => &[1,2,3,4,4,5,6],
            "ffff" => &[5,5,6,7,7,8,9]
        ).unwrap();
        let a = df.filter(&df.column("asdf").unwrap().is_null()).unwrap();
        let b = df.filter(&df.column("asdf").unwrap().is_not_null()).unwrap();
        dbg!(a);
        dbg!(b);
    }

    pub async fn get_client() -> Client<Compat<TcpStream>> {
        rssql::utils::get_client("username", "password", "host", "database").await
    }

    #[derive(ORM, Debug, Default)]
    #[rssql(table = CUSTOMER_LIST, schema = MASTER_DATA)]
    pub struct Customerlist {
        pub(crate) ship_to_id: Option<String>,
        #[rssql(foreign_key = "SLOW_MOVING.stock_in_day")]
        pub(crate) ship_to: Option<String>,
        pub(crate) volume: Option<i32>,
        pub(crate) container: Option<String>,
    }

    #[derive(ORM, Debug, Default)]
    #[rssql(table = SLOW_MOVING)]
    pub struct Test {
        pub(crate) stock_in_day: Option<String>,
        pub(crate) total_value: Option<f64>,
        pub(crate) Week: Option<i64>,
        // pub(crate) Generated_Time: Option<NaiveDateTime>,
    }

    #[derive(ORM, Debug, Default)]
    #[rssql(table = Person)]
    pub struct Person {
        pub(crate) id: i32,
        pub(crate) Email: String,
    }

    #[derive(ORM, Debug, Default)]
    #[rssql(table = FORECAST)]
    pub struct Fcst {
        pub(crate) Customer: Option<String>,
        pub(crate) Material: Option<String>,
        Dv: Option<f64>,
        Route: Option<String>,
        TransitTime: Option<String>,
        Plant: Option<String>,
    }

    // #[derive(ORM, Debug, Default)]
    // #[rssql(table = SA)]
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


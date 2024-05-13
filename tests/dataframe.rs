#![allow(non_snake_case)]
#![cfg(feature = "polars")]

use tokio::net::TcpStream;
use tokio_util::compat::Compat;

use ssql::prelude::*;
use tiberius::Client;

#[cfg(feature = "polars")]
#[tokio::test]
async fn data_frame() {
    let mut client = get_client().await;
    let df = Person::query().df(&mut client).await.unwrap();
    dbg!(df);
}

pub async fn get_client() -> Client<Compat<TcpStream>> {
    ssql::utils::get_client("username", "password", "host", "database").await
}

#[derive(ORM, Debug, Default)]
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
pub struct Test {
    pub(crate) stock_in_day: Option<String>,
    pub(crate) total_value: Option<f64>,
    pub(crate) Week: Option<i64>,
    // pub(crate) Generated_Time: Option<NaiveDateTime>,
}

#[derive(ORM, Debug, Default)]
#[ssql(table = Person)]
pub struct Person {
    pub(crate) id: i32,
    pub(crate) Email: String,
}

#[derive(ORM, Debug, Default)]
#[ssql(table = FORECAST, schema = UPDATED_DATA)]
pub struct Fcst {
    pub(crate) Customer: Option<String>,
    pub(crate) Material: Option<String>,
    Dv: Option<f64>,
    Route: Option<String>,
    TransitTime: Option<String>,
    Plant: Option<String>,
}

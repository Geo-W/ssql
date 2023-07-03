mod structs;
pub mod prelude;

use tokio;


#[cfg(test)]
mod tests {
    use tiberius::{AuthMethod, Client, Config};
    use tokio::net::TcpStream;
    use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};
    use super::*;
    use prelude::*;

    #[tokio::test]
    async fn it_works() {
        let client = get_client().await;
        dbg!(&client);
        let query = Customerlist::query()
            .join::<Test>();
        dbg!("{:?}",query.find_all_test().await);
    }

    #[test]
    fn test() {
        let query = Customerlist::query()
            .join::<Test>();
        dbg!(&query.fields);
    }


    pub async fn get_client() -> Client<Compat<TcpStream>> {
        let mut config = Config::new();
        config.host("?");
        // config.port(8080);
        config.authentication(AuthMethod::sql_server("?", "?"));
        config.trust_cert(); // on production, it is not a good idea to do this
        config.database("?");
        let tcp = TcpStream::connect(config.get_addr()).await.unwrap();
        tcp.set_nodelay(true).unwrap();
        let mut client = Client::connect(config, tcp.compat_write()).await.unwrap();
        client
    }

    #[derive(ORM, Debug, Default)]
    #[rusql(table = CUSTOMER_LIST)]
    pub struct Customerlist {
        pub(crate) ship_to_id: String,
        #[rusql(foreign_key = "SLOW_MOVING.stock_in_day")]
        pub(crate) ship_to: String,
        pub(crate) volume: i32,
        pub(crate) container: String,
    }

    #[derive(ORM, Debug, Default)]
    #[rusql(table = SLOW_MOVING)]
    pub struct Test {
        pub(crate) stock_in_day: String,
        pub(crate) total_value: f64,
        pub(crate) Week: i64,
        pub(crate) Generated_Time: String
    }


}



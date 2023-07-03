use std::collections::HashMap;
use serde_json::{Map, Value};
use tiberius::{AuthMethod, Client, Config, QueryItem};
use tokio::net::TcpStream;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};
use futures_lite::stream::StreamExt;

pub struct QueryBuilder {
    table: &'static str,
    pub(crate) fields: HashMap<&'static str, Vec<&'static str>>,
    pub(crate) filters: Vec<String>,
    pub(crate) join: String,
    sql: String,
    relation_func: Box<dyn Fn(&str) -> &'static str>,
    from_row_funcs: HashMap<String, Box<dyn Fn(&tiberius::Row) -> Map<String, Value>>>, // mapper from table name to select row func
}

impl QueryBuilder {
    pub fn new(table: &'static str, fields: (&'static str, Vec<&'static str>), func: Box<dyn Fn(&str) -> &'static str>, from_row: Box<dyn Fn(&tiberius::Row) -> Map<String, Value>>) -> Self {
        QueryBuilder {
            table: table,
            fields: HashMap::from([fields]),
            filters: vec![],
            join: String::new(),
            relation_func: func,
            sql: "".to_string(),
            from_row_funcs: HashMap::from([(table.to_string(), from_row)]),
        }
    }

    pub fn filter(mut self, field: &str, condition: impl ToString) -> Self {
        self.filters.push(format!("{}{}", field, condition.to_string()));
        self
    }

    pub fn join<T>(mut self) -> Self
        where T: RusqlMarker {
        let name = T::table_name();
        let fields = T::fields();
        println!("name: {:?}", name);
        let relation = self.find_relation(&name);
        self.join.push_str(&format!(" LEFT JOIN {} ", relation));
        match self.fields.insert(&name, fields) {
            Some(_v) => panic!("table already joined."),
            None => {}
        }
        self
    }

    fn find_relation(&self, table: &str) -> &'static str {
        (self.relation_func)(table)
    }

    pub fn raw(mut self, sql: &str) -> Self {
        self.sql = sql.to_string();
        self
    }

    pub async fn find_all(&self, mut conn: tiberius::Client<Compat<TcpStream>>) -> Vec<Value> {
        let mut sql = self.fields.iter()
            .map(|(table, fields)|
                fields.iter().map(|field| format!(r#"{}.{} as "{}.{}""#, table, field, table, field))
                    .reduce(|cur, nxt| format!("{},{}", cur, nxt)).unwrap()
            )
            .reduce(|cur, nxt| format!("{},{}", cur, nxt)).unwrap();
        // let mut stream = conn.simple_query(r#"SELECT ship_to_id as "CUSTOMER_LIST.ship_to_id", ship_to as "CUSTOMER_LIST.ship_to",
        // volume as "CUSTOMER_LIST.volume", container as "CUSTOMER_LIST.container" FROM CUSTOMER_LIST"#).await.unwrap();
        dbg!(format!("SELECT {} FROM {} {} ", sql, self.table, self.join));
        let mut stream = conn.simple_query(format!("SELECT {} FROM {} {} ", sql, self.table, self.join)).await.unwrap();
        let mut ret = Vec::new();
        while let Some(item) = stream.try_next().await.unwrap() {
            match item {
                QueryItem::Row(row) => {
                    dbg!(&row);
                    ret.push((self.from_row_funcs.get(self.table).unwrap())(&row).into());
                }
                QueryItem::Metadata(_) => {}
            }
        }
        ret
    }

    pub async fn get_client() -> Client<Compat<TcpStream>> {
        let mut config = Config::new();
        config.host("sgpvm00529.apac.bosch.com");
        // config.port(8080);
        config.authentication(AuthMethod::sql_server("biadmin", "biadmin"));
        config.trust_cert(); // on production, it is not a good idea to do this
        config.database("DB_Don_BIDATA_SQL");
        let tcp = TcpStream::connect(config.get_addr()).await.unwrap();
        tcp.set_nodelay(true).unwrap();
        let mut client = Client::connect(config, tcp.compat_write()).await.unwrap();
        client
    }

    pub async fn find_all_test(&self) -> Vec<Value> {
        let client = QueryBuilder::get_client().await;
        self.find_all(client).await
    }
}


pub trait RusqlMarker: Sized {
    fn table_name() -> &'static str;
    fn fields() -> Vec<&'static str>;
}

struct TableInfo<RusqlMarker> {
    table_name: String,
    relationship: Box<dyn Fn(&str) -> &'static str>,
    from_row: Box<dyn Fn(tiberius::Row) -> RusqlMarker>,
}
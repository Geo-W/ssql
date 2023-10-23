use tiberius::Client;
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

/// Getting client, only for testing purpose.
pub async fn get_client(
    username: &str,
    password: &str,
    host: &str,
    database: &str,
) -> tiberius::Client<tokio_util::compat::Compat<TcpStream>> {
    let mut config = tiberius::Config::new();
    config.host(host);
    // config.port(8080);
    config.authentication(tiberius::AuthMethod::sql_server(username, password));
    config.trust_cert(); // on production, it is not a good idea to do this
    config.database(database);
    let tcp = TcpStream::connect(config.get_addr()).await.unwrap();
    tcp.set_nodelay(true).unwrap();
    let client = Client::connect(config, tcp.compat_write()).await.unwrap();
    client
}

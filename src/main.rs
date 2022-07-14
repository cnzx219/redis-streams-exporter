mod routes;

use clap::Parser;
use lazy_static::lazy_static;
use tracing::info;
use std::net::SocketAddr;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {

    #[clap(short = 'k', long = "key", default_value = "")]
    key: String,

    #[clap(short = 'r', long = "redis", default_value = "redis://127.0.0.1/0")]
    redis: String,

    #[clap(short = 'd', long = "redis-db", default_value = "0")]
    db: String,

    #[clap(short = 'b', long = "bind", default_value = "127.0.0.1:9219")]
    bind: String,

}

lazy_static! {
    static ref CONFIG: Args = Args::parse();
}

fn init_logger() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info")
    }
    tracing_subscriber::fmt::init();
}

async fn init_server() {
    let routes = routes::build_routes().await;
    let addr = SocketAddr::from_str(&CONFIG.bind).unwrap();
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(routes.into_make_service())
        .await
        .unwrap();
}

#[tokio::main]
async fn main() {
    init_logger();
    info!("Hello, world!");
    init_server().await;
}

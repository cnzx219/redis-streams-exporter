use clap::Parser;
use lazy_static::lazy_static;
use tracing::info;
use std::net::SocketAddr;
use std::str::FromStr;
use axum::{
    routing::{get},
    Router,
    http::StatusCode,
    response::IntoResponse,
};
use redis::Commands;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {

    #[clap(short = 'k', long = "key", default_value = "trades_coinbase")]
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

    pub static ref REDIS: Mutex<redis::Connection> = {
        let redis_lient = redis::Client::open(CONFIG.redis.to_owned()).unwrap();
        let redis_conn = redis_lient.get_connection().unwrap();
        Mutex::new(redis_conn)
    };

    pub static ref STREAM_KEY: String = {
        CONFIG.key.to_owned()
    };
}

async fn index() -> impl IntoResponse {
    (StatusCode::OK, axum::response::Html("<html><head><title>Redis Streams Exporter</title></head><body><h1>Redis Streams Exporter</h1><p><a href=\"/metrics\">Metrics</a></p></body></html>"))
}

async fn metrics() -> impl IntoResponse {
    // let mut conn = REDIS.lock().await;
    let mut conn = REDIS.lock().unwrap();
    let stream_info_reply: redis::streams::StreamInfoStreamReply = conn.xinfo_stream(STREAM_KEY.to_owned()).unwrap();
    let length_str = format!("{}", stream_info_reply.length);
    let group_count_str = format!("{}", stream_info_reply.groups);
    let mut strs: Vec<String> = Vec::new();
    strs.push(String::from(r###"
# HELP redis_stream_length Number of messages in the stream
# TYPE redis_stream_length gauge
redis_stream_length\{stream="my-stream-key"\} "###));
    strs.push(length_str);
    strs.push(String::from("\n\n"));
    strs.push(String::from(r###"
# HELP redis_stream_earliest_id The epoch timestamp of the earliest message on the stream
# TYPE redis_stream_earliest_id gauge
redis_stream_earliest_id{stream="my-stream-key"} "###));
    strs.push(String::from(r###"
# HELP redis_stream_latest_id The epoch timestamp of the latest message on the stream
# TYPE redis_stream_latest_id gauge
redis_stream_latest_id{stream="my-stream-key"} "###));
    strs.push(stream_info_reply.first_entry.id);
    strs.push(String::from("\n\n"));
    strs.push(String::from(r###"
# HELP redis_stream_earliest_id The epoch timestamp of the earliest message on the stream
# TYPE redis_stream_earliest_id gauge
redis_stream_earliest_id{stream="my-stream-key"} "###));
    strs.push(stream_info_reply.last_entry.id);
    strs.push(String::from("\n\n"));
    strs.push(String::from(r###"
# HELP redis_stream_consumer_groups_total Number of consumer groups for the stream
# TYPE redis_stream_consumer_groups_total gauge
redis_stream_consumer_groups_total{stream="my-stream-key"} "###));
    strs.push(group_count_str);
    strs.push(String::from("\n\n"));
    strs.push(String::from(r###"
# HELP redis_stream_last_generated_id Number of consumer groups for the stream
# TYPE redis_stream_last_generated_id gauge
redis_stream_last_generated_id{stream="my-stream-key"} "###));
    strs.push( stream_info_reply.last_generated_id);
    strs.push(String::from("\n\n"));

    (StatusCode::OK, strs.join(""))
}

async fn init_server() {
    let routes = Router::new()
        .route("/", get(index))
        .route("/metrics", get(metrics));
    let addr = SocketAddr::from_str(&CONFIG.bind).unwrap();
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(routes.into_make_service())
        .await
        .unwrap();
}

fn init_logger() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info")
    }
    tracing_subscriber::fmt::init();
}

#[tokio::main]
async fn main() {
    init_logger();
    lazy_static::initialize(&REDIS);
    init_server().await;
}

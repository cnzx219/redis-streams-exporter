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
use tokio::sync::Mutex;
use std::time::SystemTime;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {

    #[clap(short = 'k', long = "key", require_equals = true)]
    key: String,

    #[clap(short = 'r', long = "redis", default_value = "redis://127.0.0.1/0")]
    redis: String,

    #[clap(short = 'b', long = "bind", default_value = "127.0.0.1:9219")]
    bind: String,

}

lazy_static! {
    static ref CONFIG: Args = Args::parse();

    static ref REDIS: Mutex<redis::Connection> = {
        let redis_lient = redis::Client::open(CONFIG.redis.to_owned()).unwrap();
        let redis_conn = redis_lient.get_connection().unwrap();
        Mutex::new(redis_conn)
    };

    static ref STREAM_KEYS: Vec<String> = {
        CONFIG.key.split(";").map(|str| String::from(str)).collect::<Vec<String>>()
    };
}

trait PairToString {
    fn to_str(&self, name: &str) -> String;
}

struct SV {
    stream: String,
    value: String,
}

impl PairToString for SV {
    fn to_str(&self, name: &str) -> String {
        format!("{}{{stream={}}} {}", name, self.stream, self.value)
    }
}

struct SGV {
    stream: String,
    group: String,
    value: String,
}

impl PairToString for SGV {
    fn to_str(&self, name: &str) -> String {
        format!("{}{{stream={},group={}}} {}", name, self.stream, self.group, self.value)
    }
}

struct SGCV {
    stream: String,
    group: String,
    consumer: String,
    value: String,
}

impl PairToString for SGCV {
    fn to_str(&self, name: &str) -> String {
        format!("{}{{stream={},group={},consumer={}}} {}", name, self.stream, self.group, self.consumer, self.value)
    }
}

struct Metrics {
    redis_stream_length: Vec<SV>,
    redis_stream_earliest_id: Vec<SV>,
    redis_stream_latest_id: Vec<SV>,
    redis_stream_consumer_groups_total: Vec<SV>,
    redis_stream_idle: Vec<SV>,

    redis_stream_consumer_group_last_delivered_id: Vec<SGV>,
    redis_stream_consumer_group_pending_messages_total: Vec<SGV>,
    redis_stream_consumer_group_consumers_total: Vec<SGV>,

    redis_stream_consumer_pending_messages_total: Vec<SGCV>,
    redis_stream_consumer_idle_time_seconds: Vec<SGCV>,
}

fn sv_quick_join<T: PairToString>(v: &Vec<T>, name: &str) -> String {
    if v.len() == 0 {
        return String::from("")
    }
    let mut strs: Vec<String> = Vec::new();
    for sv in v {
        strs.push(sv.to_str(name));
    }
    strs.join("\n")
}

impl Metrics {
    fn to_str(&self) -> String {
        let mut strs: Vec<String> = Vec::new();
        // steam
        strs.push(String::from("# HELP redis_stream_length Number of messages in the stream\n# TYPE redis_stream_length gauge"));
        strs.push(sv_quick_join(&self.redis_stream_length, "redis_stream_length"));
        strs.push(String::from("# HELP redis_stream_earliest_id The epoch timestamp of the earliest message on the stream\n# TYPE redis_stream_earliest_id gauge"));
        strs.push(sv_quick_join(&self.redis_stream_earliest_id, "redis_stream_earliest_id"));
        strs.push(String::from("# HELP redis_stream_latest_id The epoch timestamp of the latest message on the stream\n# TYPE redis_stream_latest_id gauge"));
        strs.push(sv_quick_join(&self.redis_stream_latest_id, "redis_stream_latest_id"));
        strs.push(String::from("# HELP redis_stream_consumer_groups_total Number of consumer groups for the stream\n# TYPE redis_stream_consumer_groups_total gauge"));
        strs.push(sv_quick_join(&self.redis_stream_consumer_groups_total, "redis_stream_consumer_groups_total"));
        strs.push(String::from("# HELP redis_stream_idle The epoch timestamp of producer idle on the stream\n# TYPE redis_stream_idle gauge"));
        strs.push(sv_quick_join(&self.redis_stream_idle, "redis_stream_idle"));
        // group
        strs.push(String::from("# HELP redis_stream_consumer_group_last_delivered_id The epoch timestamp of the last delivered message\n# TYPE redis_stream_consumer_group_last_delivered_id gauge"));
        strs.push(sv_quick_join(&self.redis_stream_consumer_group_last_delivered_id, "redis_stream_consumer_group_last_delivered_id"));
        strs.push(String::from("# HELP redis_stream_consumer_group_pending_messages_total Number of pending messages for the group\n# TYPE redis_stream_consumer_group_pending_messages_total gauge"));
        strs.push(sv_quick_join(&self.redis_stream_consumer_group_pending_messages_total, "redis_stream_consumer_group_pending_messages_total"));
        strs.push(String::from("# HELP redis_stream_consumer_group_consumers_total Number of consumers in the group\n# TYPE redis_stream_consumer_group_consumers_total gauge"));
        strs.push(sv_quick_join(&self.redis_stream_consumer_group_consumers_total, "redis_stream_consumer_group_consumers_total"));
        // consumer
        strs.push(String::from("# HELP redis_stream_consumer_pending_messages_total Number of pending messages for the consumer\n# TYPE redis_stream_consumer_pending_messages_total gauge"));
        strs.push(sv_quick_join(&self.redis_stream_consumer_pending_messages_total, "redis_stream_consumer_pending_messages_total"));
        strs.push(String::from("# HELP redis_stream_consumer_idle_time_seconds The amount of time for which the consumer has been idle\n# TYPE redis_stream_consumer_idle_time_seconds gauge"));
        strs.push(sv_quick_join(&self.redis_stream_consumer_idle_time_seconds, "redis_stream_consumer_idle_time_seconds"));
        strs.join("\n")
    }
}

async fn get_metrics() -> Metrics {
    let mut ret = Metrics{
        redis_stream_length: Vec::new(),
        redis_stream_earliest_id: Vec::new(),
        redis_stream_latest_id: Vec::new(),
        redis_stream_consumer_groups_total: Vec::new(),
        redis_stream_idle: Vec::new(),

        redis_stream_consumer_group_last_delivered_id: Vec::new(),
        redis_stream_consumer_group_pending_messages_total: Vec::new(),
        redis_stream_consumer_group_consumers_total: Vec::new(),

        redis_stream_consumer_pending_messages_total: Vec::new(),
        redis_stream_consumer_idle_time_seconds: Vec::new(),
    };

    let mut conn = REDIS.lock().await;
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
    let stream_keys: &Vec<String> = &STREAM_KEYS;
    for stream_key in stream_keys {
        let stream_info_reply: redis::streams::StreamInfoStreamReply = conn.xinfo_stream(stream_key.to_owned()).unwrap();
        ret.redis_stream_length.push(SV { stream: stream_key.to_string(), value: stream_info_reply.length.to_string() });
        ret.redis_stream_earliest_id.push(SV {stream: stream_key.to_string(), value: stream_info_reply.first_entry.id});
        ret.redis_stream_latest_id.push(SV {stream: stream_key.to_string(), value: stream_info_reply.last_entry.id});
        ret.redis_stream_consumer_groups_total.push(SV {stream: stream_key.to_string(), value: stream_info_reply.groups.to_string()});

        let splited_id = stream_info_reply.last_generated_id.split("-").collect::<Vec<&str>>();
        let id = splited_id.get(0).unwrap().parse::<u128>().unwrap();
        let gap = (now - id) / 1000;
        ret.redis_stream_idle.push(SV {stream: stream_key.to_string(), value: gap.to_string()});

        let groups_info_reply: redis::streams::StreamInfoGroupsReply = conn.xinfo_groups(stream_key.to_owned()).unwrap();
        for group in &groups_info_reply.groups {
            ret.redis_stream_consumer_group_last_delivered_id.push(SGV {stream: stream_key.to_string(), group: group.name.to_owned(), value: group.last_delivered_id.to_owned() });
            ret.redis_stream_consumer_group_pending_messages_total.push(SGV {stream: stream_key.to_string(), group: group.name.to_owned(), value: group.pending.to_string() });
            ret.redis_stream_consumer_group_consumers_total.push(SGV {stream: stream_key.to_string(), group: group.name.to_owned(), value: group.consumers.to_string() });

            let consumers_info_reply: redis::streams::StreamInfoConsumersReply = conn.xinfo_consumers(stream_key.to_owned(), group.name.to_owned()).unwrap();
            for consumer_ in &consumers_info_reply.consumers {
                ret.redis_stream_consumer_pending_messages_total.push(SGCV {stream: stream_key.to_string(), group: group.name.to_owned(), consumer: consumer_.name.to_string(), value: consumer_.pending.to_string() });
                ret.redis_stream_consumer_idle_time_seconds.push(SGCV {stream: stream_key.to_string(), group: group.name.to_owned(), consumer: consumer_.name.to_string(), value: consumer_.idle.to_string() });
            }
        }

    }

    ret
}

async fn response_metrics() -> impl IntoResponse {
    let metrics = get_metrics().await;
    (StatusCode::OK, metrics.to_str())
}

async fn response_index() -> impl IntoResponse {
    (StatusCode::OK, axum::response::Html("<html><head><title>Redis Streams Exporter</title></head><body><h1>Redis Streams Exporter</h1><p><a href=\"/metrics\">Metrics</a></p></body></html>"))
}

async fn init_server() {
    let routes = Router::new()
        .route("/", get(response_index))
        .route("/metrics", get(response_metrics));
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

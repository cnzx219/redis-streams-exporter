use axum::{
    routing::{get, post},
    Router,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

pub async fn build_routes() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/metrics", get(metrics))
        .route("/-/reload", post(reload))
}

#[derive(Serialize)]
struct DataVO {
    data: String,
}

async fn index() -> &'static str {
    "<html><head><title>Redis Streams Exporter</title></head><body><h1>Node Exporter</h1><p><a href=\"/metrics\">Metrics</a></p></body></html>"
}

async fn metrics() -> impl IntoResponse {
    (StatusCode::OK, String::from("hello"))
}

async fn reload() -> impl IntoResponse {
    let data_vo = DataVO { data: String::from("hello") };
    (StatusCode::OK, Json(data_vo))
}

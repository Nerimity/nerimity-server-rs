mod api_error;
mod auth;
mod cache;
mod db;
mod jwt;
mod proto;
mod routes;
mod services;

use axum::{
    Json, Router,
    body::to_bytes,
    http::{StatusCode, header},
    middleware,
    response::{IntoResponse, Response},
};
use jsonwebtoken::DecodingKey;
use redis::aio::ConnectionManager;
use serde_json::json;
use tonic::transport::Channel;

use crate::proto::{make_socket_client, pb::socket_client::SocketClient};

#[derive(Clone)]
struct AppState {
    socket: SocketClient<Channel>,
    db: sqlx::PgPool,
    jwt_decode_key: DecodingKey,
    redis: ConnectionManager,
}

async fn json_errors(res: Response) -> Response {
    let status = res.status();

    let is_plain_text = res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("text/plain"));

    if (status.is_client_error() || status.is_server_error()) && is_plain_text {
        let (_, body) = res.into_parts();
        let bytes = to_bytes(body, 64 * 1024).await.unwrap_or_default();
        let message = String::from_utf8_lossy(&bytes).to_string();
        return (status, Json(json!({ "message": message }))).into_response();
    }

    res
}

async fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "message": "not found" })),
    )
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

    println!("[Redis] Connecting...");
    let redis_client = redis::Client::open(redis_url).unwrap();
    let redis = ConnectionManager::new(redis_client).await.unwrap();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("[DB] Connecting...");
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .unwrap();

    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let state = AppState {
        socket: make_socket_client(),
        jwt_decode_key: DecodingKey::from_secret(jwt_secret.as_bytes())
            .expect("invalid JWT secret"),
        db,
        redis,
    };

    let app = Router::new()
        .nest("/api", routes::message_routes::router())
        .fallback(not_found)
        .layer(middleware::map_response(json_errors))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!("http://localhost:3001");
    axum::serve(listener, app).await.unwrap();
}

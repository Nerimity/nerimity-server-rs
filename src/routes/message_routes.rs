use std::net::SocketAddr;

use axum::{
    Json, Router,
    extract::{ConnectInfo, Path, State},
    http::HeaderMap,
    routing::post,
};
use serde::Deserialize;

use crate::{
    AppState, api_error::ApiError, auth::auth_from_headers, cache::user_cache_repo::UserCache,
    services::message_service,
};

#[derive(Deserialize)]
struct CreateMessageRequest {
    content: String,
}

impl CreateMessageRequest {
    fn validate(&self) -> Result<(), ApiError> {
        if self.content.trim().is_empty() {
            return Err(ApiError::unprocessable("content is required"));
        }
        Ok(())
    }
}

async fn create_message(
    Path(_channel_id): Path<String>,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<CreateMessageRequest>,
) -> Result<Json<UserCache>, ApiError> {
    body.validate()?;

    let res = auth_from_headers(&headers, &addr, &state).await?;

    // message_service::create_message(&state, channel_id.clone()).await?;
    // Ok(channel_id)
    Ok(Json(res))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/channels/{channel_id}/messages", post(create_message))
}

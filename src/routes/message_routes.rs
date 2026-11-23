use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    routing::post,
};
use serde::Deserialize;

use crate::{AppState, api_error::ApiError, auth::auth_from_headers, services::message_service};

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
    Path(channel_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateMessageRequest>,
) -> Result<String, ApiError> {
    body.validate()?;

    let res = auth_from_headers(&headers, &state).await?;

    println!("cache: {:?}", res);

    message_service::create_message(&state, channel_id.clone()).await?;
    Ok(channel_id)
}

pub fn router() -> Router<AppState> {
    Router::new().route("/channels/{channel_id}/messages", post(create_message))
}

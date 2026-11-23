use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};
use serde::{Deserialize, Serialize};

use crate::{AppState, auth::auth_from_headers, services::message_service};

#[derive(Serialize)]
struct ErrorBody {
    message: String,
}

pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn unprocessable(message: &str) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: message.to_string(),
        }
    }
    pub fn not_found(message: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.to_string(),
        }
    }
    pub fn unauthorized(message: &str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.to_string(),
        }
    }
    fn internal_server_error(message: &str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.to_string(),
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(_: sqlx::Error) -> Self {
        Self::internal_server_error("Database error")
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                message: self.message,
            }),
        )
            .into_response()
    }
}

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

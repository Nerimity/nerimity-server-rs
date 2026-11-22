use axum::http::HeaderMap;

use crate::{AppState, jwt::decode_string_jwt, routes::message_routes::ApiError};

pub async fn auth(token: &str, state: &AppState) -> Result<(), ApiError> {
    let missing_token_err = || ApiError::unauthorized("Missing Token");

    if token.is_empty() {
        return Err(ApiError::unauthorized("Missing Token"));
    }

    let decoded_token = decode_string_jwt(token, &state.jwt_decode_key)?;

    let mut parts = decoded_token.split("-");

    let Some(session_id) = parts.next() else {
        return Err(missing_token_err());
    };

    println!("{session_id}");

    Ok(())
}
pub async fn auth_from_headers(headers: &HeaderMap, state: &AppState) -> Result<(), ApiError> {
    let missing_token_err = || ApiError::unauthorized("Missing Token");

    let token = headers
        .get("Authorization")
        .ok_or_else(missing_token_err)?
        .to_str()
        .map_err(|_| missing_token_err())?;

    auth(token, state).await
}

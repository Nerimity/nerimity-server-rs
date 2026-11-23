use axum::http::HeaderMap;

use crate::{
    AppState,
    cache::user_cache_repo::{UserCache, get_user_cache_by_session_id},
    jwt::decode_string_jwt,
    routes::message_routes::ApiError,
};

pub async fn auth(token: &str, state: &AppState) -> Result<UserCache, ApiError> {
    let missing_token_err = || ApiError::unauthorized("Missing Token");
    let invalid_token_err = || ApiError::unauthorized("Invalid Token");

    if token.is_empty() {
        return Err(ApiError::unauthorized("Missing Token"));
    }

    let decoded_token = decode_string_jwt(token, &state.jwt_decode_key)?;

    let mut parts = decoded_token.split("-");

    let Some(session_id) = parts.next() else {
        return Err(missing_token_err());
    };

    let user_cache_result = get_user_cache_by_session_id(&state.redis, session_id).await;

    let maybe_user_cache = user_cache_result.map_err(|_| invalid_token_err())?;

    match maybe_user_cache {
        Some(user_cache) => Ok(user_cache),
        None => Err(invalid_token_err()),
    }
}
pub async fn auth_from_headers(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<UserCache, ApiError> {
    let missing_token_err = || ApiError::unauthorized("Missing Token");

    let token = headers
        .get("Authorization")
        .ok_or_else(missing_token_err)?
        .to_str()
        .map_err(|_| missing_token_err())?;

    auth(token, state).await
}

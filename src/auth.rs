use axum::http::HeaderMap;

use crate::{
    AppState,
    api_error::ApiError,
    cache::user_cache_repo::{UserCache, get_user_cache_by_session_id},
    db::{
        user_device_repo::{self},
        user_repo,
    },
    jwt::decode_string_jwt,
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
    let maybe_user_cache = user_cache_result
        .map_err(|_| ApiError::internal_server_error("Failed to authenticate. Try again later."))?;

    match maybe_user_cache {
        Some(user_cache) => return Ok(user_cache),
        _ => {}
    };

    // database

    println!("datebase checking...");

    let user_device = user_device_repo::find_by_session_id(&state.db, session_id.to_string())
        .await?
        .ok_or_else(|| invalid_token_err())?;

    let user = user_repo::get_user_with_options(
        &state.db,
        &user_device.user_id,
        user_repo::UserQueryOptions {
            include_account: true,
            include_private: true,
        },
    )
    .await?
    .unwrap();

    println!("{:?}", user);

    return Err(invalid_token_err());
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

use std::net::SocketAddr;

use axum::http::HeaderMap;

use crate::{
    AppState,
    api_error::ApiError,
    cache::user_cache_repo::{
        AccountCache, ApplicationCache, UserCache, get_user_cache_by_session_id, store_user_cache,
    },
    db::{
        user_device_repo::{self},
        user_repo,
    },
    http_utils::client_info,
    jwt::decode_string_jwt,
};

pub struct AuthOptions<'a> {
    state: &'a AppState,
    token: Option<&'a str>,
    addr: Option<&'a SocketAddr>,
    headers: Option<&'a HeaderMap>,
}
pub async fn auth(opts: AuthOptions<'_>) -> Result<UserCache, ApiError> {
    let missing_token_err = || ApiError::unauthorized("Missing Token");
    let invalid_token_err = || ApiError::unauthorized("Invalid Token");

    let token = opts
        .token
        .or_else(|| opts.headers?.get("Authorization")?.to_str().ok())
        .ok_or_else(missing_token_err)?;

    let decoded_token = decode_string_jwt(token, &opts.state.jwt_decode_key)?;

    let mut parts = decoded_token.split("-");

    let Some(session_id) = parts.next() else {
        return Err(missing_token_err());
    };

    let user_cache_result = get_user_cache_by_session_id(&opts.state.redis, session_id).await;
    let maybe_user_cache = user_cache_result
        .map_err(|_| ApiError::internal_server_error("Failed to authenticate. Try again later."))?;

    match maybe_user_cache {
        Some(user_cache) => return Ok(user_cache),
        _ => {}
    };

    // database

    if let (Some(headers), Some(addr)) = (opts.headers, opts.addr) {
        let info = client_info(&headers, *addr);

        println!("{:?}", info);
    }

    println!("database checking...");

    let user_device = user_device_repo::find_by_session_id(&opts.state.db, session_id.to_string())
        .await?
        .ok_or_else(|| invalid_token_err())?;

    let user = user_repo::find_for_cache_by_id(&opts.state.db, &user_device.user_id)
        .await?
        .ok_or_else(|| invalid_token_err())?;

    let cache = UserCache {
        username: user.username,
        tag: user.tag,
        avatar: user.avatar,
        banner: user.banner,
        badges: user.badges,
        bot: user.bot,
        hex_color: user.hex_color,
        id: user.id,
        ip: user.ip,
        shadow_banned: user.shadow_banned,
        application: user.application.map(|a| ApplicationCache { id: a.id }),
        account: user.account.map(|a| AccountCache {
            id: a.id,
            email_confirmed: a.email_confirmed,
        }),
    };

    store_user_cache(&opts.state.redis, &cache, session_id.to_string())
        .await
        .map_err(|_| invalid_token_err())?;

    return Ok(cache);
}

pub async fn auth_from_headers(
    headers: &HeaderMap,
    addr: &SocketAddr,
    state: &AppState,
) -> Result<UserCache, ApiError> {
    auth(AuthOptions {
        token: None,
        state,
        addr: Some(addr),
        headers: Some(headers),
    })
    .await
}

use std::{error::Error, sync::LazyLock};

use redis::{Script, aio::ConnectionManager};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserCache {
    pub id: String,
    pub username: String,
    pub hex_color: Option<String>,
    pub tag: String,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub badges: u32,
    pub bot: Option<bool>,
    pub account: Option<AccountCache>,
    pub application: Option<ApplicationCache>,
    pub ip: Option<String>,
    pub shadow_banned: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountCache {
    pub id: String,
    pub email_confirmed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationCache {
    pub id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("failed to parse cached value: {0}")]
    Parse(#[from] serde_json::Error),
}

const PREFIX: &str = "NERIMITY_";

fn user_cache_key(user_id: &str) -> String {
    format!("{PREFIX}USER:{user_id}")
}

fn session_id_to_user_id_key(session_id: &str) -> String {
    format!("{PREFIX}SESSION_USER_ID:{session_id}")
}

const LUA_SESSION_FETCH: &str = r#"
    local user_id = redis.call("GET", KEYS[1])
    if not user_id then
      return nil
    end

    local user_cache_prefix = ARGV[1]
    return redis.call("GET", user_cache_prefix .. user_id)
"#;

static SESSION_FETCH: LazyLock<Script> = LazyLock::new(|| Script::new(LUA_SESSION_FETCH));

pub async fn get_user_cache_by_session_id(
    redis: &ConnectionManager,
    session_id: &str,
) -> Result<Option<UserCache>, CacheError> {
    let mut conn = redis.clone();

    let raw: Option<String> = SESSION_FETCH
        .key(session_id_to_user_id_key(session_id))
        .arg(user_cache_key(""))
        .invoke_async(&mut conn)
        .await?;

    match raw {
        Some(json) => Ok(Some(serde_json::from_str(&json)?)),
        None => Ok(None),
    }
}

const HOUR_TO_SECS: u64 = 3600;

pub async fn store_user_cache(
    redis: &ConnectionManager,
    cache: &UserCache,
    session_id: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut conn = redis.clone();

    let json = serde_json::to_string(&cache)?;

    redis::pipe()
        .atomic()
        .set_ex(
            session_id_to_user_id_key(&session_id),
            &cache.id,
            HOUR_TO_SECS,
        )
        .set_ex(user_cache_key(&cache.id), json, HOUR_TO_SECS)
        .query_async::<()>(&mut conn)
        .await?;
    Ok(())
}

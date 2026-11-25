use serde::{Deserialize, Serialize};
use sqlx::{PgPool, types::Json};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub username: String,
    pub hex_color: Option<String>,
    pub tag: String,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub badges: u32,
    pub bot: Option<bool>,
    pub account: Option<Account>,
    pub application: Option<Application>,
    pub ip: Option<String>,
    pub shadow_banned: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub email_confirmed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub id: String,
}

pub async fn find_for_cache_by_id(
    pool: &PgPool,
    user_id: &str,
) -> Result<Option<User>, sqlx::Error> {
    let row: Option<Json<User>> = sqlx::query_scalar!(
        r#"
        SELECT to_jsonb(u) || jsonb_build_object(
            'account', (
                SELECT jsonb_build_object(
                    'id', a.id,
                    'emailConfirmed', a."emailConfirmed"
                )
                FROM accounts a
                WHERE a."userId" = u.id
                LIMIT 1
            ),
            'application', (
                SELECT jsonb_build_object('id', app.id)
                FROM applications app
                WHERE app."botUserId" = u.id
                LIMIT 1
            ),
            'ip', (
                SELECT ud."ipAddress"
                FROM user_devices ud
                WHERE ud."userId" = u.id
                ORDER BY ud."lastSeenAt" DESC
                LIMIT 1
            ),
            'shadowBanned', EXISTS(
                SELECT 1
                FROM shadow_bans sb
                WHERE sb."userId" = u.id
            )
        ) AS "user!: Json<User>"
        FROM users u
        WHERE u.id = $1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|Json(user)| user))
}

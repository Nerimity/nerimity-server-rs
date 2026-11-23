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

pub struct UserQueryOptions {
    pub include_account: bool,
    pub include_private: bool,
}

pub async fn get_user_with_options(
    pool: &PgPool,
    user_id: &str,
    opts: UserQueryOptions,
) -> Result<Option<User>, sqlx::Error> {
    let row: Option<Json<User>> = sqlx::query_scalar!(
        r#"
        SELECT jsonb_build_object(
            'id', u.id,
            'username', u.username,
            'hexColor', u."hexColor",
            'tag', u.tag,
            'avatar', u.avatar,
            'banner', u.banner,
            'badges', u.badges,
            'bot', u.bot,
            'account', CASE
                WHEN $2 THEN (
                    SELECT jsonb_build_object(
                        'id', a.id,
                        'emailConfirmed', a."emailConfirmed"
                    )
                    FROM accounts a
                    WHERE a."userId" = u.id
                    LIMIT 1
                )
                ELSE NULL
            END,
            'application', (
                SELECT jsonb_build_object(
                    'id', app.id
                )
                FROM applications app
                WHERE app."botUserId" = u.id
                LIMIT 1
            ),
            'ip', CASE
                WHEN $3 THEN (
                    SELECT ud."ipAddress"
                    FROM user_devices ud
                    WHERE ud."userId" = u.id
                    ORDER BY ud."lastSeenAt" DESC
                    LIMIT 1
                )
                ELSE NULL
            END,
            'shadowBanned', CASE
                WHEN $3 THEN (
                    SELECT EXISTS(
                        SELECT 1
                        FROM shadow_bans sb
                        WHERE sb."userId" = u.id
                    )
                )
                ELSE NULL
            END
        ) AS "user!: Json<User>"
        FROM users u
        WHERE u.id = $1
        "#,
        user_id,
        opts.include_account,
        opts.include_private
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|Json(user)| user))
}

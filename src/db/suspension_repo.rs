use serde::{Deserialize, Serialize};
use sqlx::{PgPool, types::Json};

use crate::db::user_repo::PublicUser;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Suspension {
    pub id: String,

    pub user_id: String,
    pub suspend_by: Option<PublicUser>,
    pub reason: Option<String>,
}

pub async fn find_by_user_id(
    pool: &PgPool,
    user_id: &str,
) -> Result<Option<Suspension>, sqlx::Error> {
    let row: Option<Json<Suspension>> = sqlx::query_scalar!(
        r#"
    SELECT to_jsonb(s) || jsonb_build_object(
        'suspendBy', (
            SELECT jsonb_build_object(
                'id', su.id,
                'username', su.username,
                'tag', su.tag,
                'avatar', su.avatar
            )
            FROM users su
            WHERE su.id = s."suspendedById"
        )
    ) AS "suspension!: Json<Suspension>"
    FROM suspensions s
    WHERE s."userId" = $1
    "#,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|Json(suspension)| suspension))
}

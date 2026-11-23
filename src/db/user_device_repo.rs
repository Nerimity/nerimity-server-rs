use serde::Serialize;
use sqlx::{PgPool, prelude::FromRow};

#[derive(Serialize, FromRow, Debug)]
#[sqlx(rename_all = "camelCase")]
pub struct UserDevice {
    pub id: String,
    pub user_id: String,
    pub session_id: Option<String>,
}

pub async fn find_by_session_id(
    pool: &PgPool,
    id: String,
) -> Result<Option<UserDevice>, sqlx::Error> {
    sqlx::query_as::<_, UserDevice>(
        r#"SELECT id, "userId", "sessionId" FROM user_devices WHERE "sessionId" = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

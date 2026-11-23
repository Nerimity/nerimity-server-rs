use serde::Serialize;
use sqlx::{PgPool, prelude::FromRow};

#[derive(Serialize, FromRow, Debug)]
#[sqlx(rename_all = "camelCase")]
pub struct Channel {
    pub id: String,
    pub name: Option<String>,
}

pub async fn find_by_id(pool: &PgPool, id: String) -> Result<Option<Channel>, sqlx::Error> {
    sqlx::query_as!(
        Channel,
        r#"SELECT id, name FROM channels WHERE id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await
}

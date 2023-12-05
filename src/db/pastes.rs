use serde::{
    Deserialize,
    Serialize,
};
use sqlx::{
    types::time::OffsetDateTime,
    FromRow,
};

/// A text paste to be retrieved and stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Paste {
    pub id: i32,
    pub user_id: Option<i32>,
    pub title: Option<String>,
    pub content: String,
    pub created_at: OffsetDateTime,
    pub expires_at: Option<OffsetDateTime>,
}

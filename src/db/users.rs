use serde::{
    Deserialize,
    Serialize,
};
use sqlx::{
    types::time::OffsetDateTime,
    FromRow,
};
use uuid::Uuid;

/// A user that can be authenticated with passkeys, stored in a PostgreSQL database.
#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    /// The ID of the user.
    pub id: i32,
    /// The UUID of the user.
    pub uuid: Uuid,
    /// The unique username of the user.
    pub username: String,
    /// When the user was created.
    pub created_at: OffsetDateTime,
    /// When the user last authenticated, if ever.
    pub last_authentication: Option<OffsetDateTime>,
}

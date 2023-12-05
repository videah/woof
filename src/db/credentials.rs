use serde::{
    Deserialize,
    Serialize,
};
use sqlx::{
    types::{
        time::OffsetDateTime,
        Json,
    },
    FromRow,
};
use uuid::Uuid;
use webauthn_rs::prelude::Passkey;

/// A user passkey credential model to be retrieved and stored in the database.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Credential {
    /// The ID of the credential.
    pub id: i32,
    /// The UUID of the user that owns this credential.
    pub user_uuid: Uuid,
    /// The raw JSON passkey credential from webauthn-rs.
    pub passkey: Json<Passkey>,
    /// When the credential was created.
    pub created_at: OffsetDateTime,
    /// When the credential was last updated.
    pub updated_at: OffsetDateTime,
}

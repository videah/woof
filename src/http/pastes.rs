use axum::{
    routing::post,
    Extension,
    Json,
    Router,
};
use serde::{
    Deserialize,
    Serialize,
};
use sqlx::types::time::OffsetDateTime;

use crate::{
    auth::passkeys::backend::AuthSession,
    db::pastes::Paste,
    http::ApiContext,
};

pub fn router() -> Router {
    Router::new().route("/api/pastes", post(create_paste))
}

/// Parameters for creating a new paste via the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPasteParams {
    title: Option<String>,
    content: String,
    expires_at: Option<OffsetDateTime>,
}

/// Create a new paste.
pub async fn create_paste(
    ctx: Extension<ApiContext>,
    auth_session: AuthSession,
    Json(paste): Json<NewPasteParams>,
) -> Json<Paste> {
    let user = auth_session.user;
    let user_id = user.map(|u| u.id);

    let paste = sqlx::query_file_as!(
        Paste,
        "sql/insert_paste.sql",
        user_id,
        paste.title,
        paste.content,
        paste.expires_at
    )
    .fetch_one(&ctx.db)
    .await
    .unwrap();

    Json(paste)
}

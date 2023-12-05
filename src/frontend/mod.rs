use axum::{
    routing::get,
    Router,
};

use crate::{
    auth::passkeys::backend::AuthSession,
    templates::{
        AuthTemplate,
        IndexTemplate,
        PasteTemplate,
    },
};

pub async fn index(session: AuthSession) -> IndexTemplate {
    IndexTemplate { user: session.user }
}

pub async fn auth() -> AuthTemplate {
    AuthTemplate
}

pub async fn paste() -> PasteTemplate {
    PasteTemplate
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/auth", get(auth))
        .route("/paste", get(paste))
}

use axum::{
    error_handling::HandleErrorLayer,
    http::StatusCode,
    response::Redirect,
    routing::{
        get,
        post,
    },
    BoxError,
    Extension,
    Router,
};

use serde::Deserialize;

use tower::ServiceBuilder;
use tower_sessions::{
    cookie::time::Duration,
    Expiry,
    MemoryStore,
    SessionManagerLayer,
};

use crate::{
    auth::passkeys::{
        authentication::{
            finish_authentication,
            start_authentication,
        },
        backend::AuthSession,
        registration::{
            finish_register,
            start_register,
        },
        PasskeyAuthState,
    },
};

pub mod passkeys;

/// Parameters passed to authentication handlers.
#[derive(Deserialize)]
pub struct AuthParams {
    /// The user's username as stored in the database.
    username: String,
}

/// Handler that clears a user's session, logging them out.
pub async fn logout(mut auth_session: AuthSession) -> Redirect {
    // If there is an error logging out, we don't care for now.
    auth_session.logout().ok();
    Redirect::temporary("/")
}

/// Defines the [Router] for the authentication API.
pub fn router() -> Router {
    let session_store = MemoryStore::default();
    let auth_service = ServiceBuilder::new()
        .layer(Extension(PasskeyAuthState::new(
            "videah-macbook.squeaker-squeaker.ts.net".to_string(),
            "https://localhost".to_string(),
        )))
        .layer(HandleErrorLayer::new(|_: BoxError| async {
            StatusCode::BAD_REQUEST
        }))
        .layer(
            SessionManagerLayer::new(session_store)
                .with_secure(false)
                .with_expiry(Expiry::OnInactivity(Duration::seconds(20))),
        );

    Router::new()
        .route("/logout", get(logout))
        .route("/api/users/start_register", post(start_register))
        .route("/api/users/finish_register", post(finish_register))
        .route(
            "/api/users/start_authentication",
            post(start_authentication),
        )
        .route(
            "/api/users/finish_authentication",
            post(finish_authentication),
        )
        .layer(auth_service)
}

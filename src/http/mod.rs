pub mod error;
pub mod pastes;

use std::sync::Arc;

use anyhow::Context;
use axum::{
    error_handling::HandleErrorLayer,
    http::StatusCode,
    BoxError,
    Extension,
    Router,
};
use axum_login::AuthManagerLayerBuilder;
use log::info;
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_sessions::{
    cookie::time::Duration,
    Expiry,
    MemoryStore,
    SessionManagerLayer,
};

use crate::{
    auth::passkeys::backend::{
        PasskeyBackend,
    },
    config::Config,
};

/// The context that is passed to all handlers to provide access to the database and configuration.
#[derive(Clone)]
pub struct ApiContext {
    pub config: Arc<Config>,
    pub db: PgPool,
}

pub async fn serve(config: Config, db: PgPool) -> anyhow::Result<()> {
    let auth_session_store = MemoryStore::default();
    let auth_session_layer = SessionManagerLayer::new(auth_session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(7)));

    let backend = PasskeyBackend::new(db.clone());

    let auth_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: BoxError| async {
            StatusCode::BAD_REQUEST
        }))
        .layer(AuthManagerLayerBuilder::new(backend, auth_session_layer).build());

    let app = api_router()
        .nest_service("/static", ServeDir::new("static"))
        .layer(auth_service)
        .layer(ServiceBuilder::new().layer(Extension(ApiContext {
            config: Arc::new(config),
            db,
        })));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;

    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .await
        .context("error running HTTP server")
}

/// Constructs the a [Router] that pulls in all the routes from the different modules.
pub fn api_router() -> Router {
    crate::auth::router()
        .merge(pastes::router())
        .merge(crate::frontend::router())
}

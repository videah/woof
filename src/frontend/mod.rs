mod paste;

use axum::{
    body::Body,
    response::{
        IntoResponse,
        Response,
    },
    routing::get,
    Router,
};
use http::StatusCode;
use thiserror::Error;

use crate::{
    auth::passkeys::backend::AuthSession,
    templates::{
        AuthTemplate,
        ErrorTemplate,
        IndexTemplate,
    },
};

/// The index page, presents a file upload form to the user.
pub async fn index(session: AuthSession) -> IndexTemplate {
    IndexTemplate { user: session.user }
}

/// The authentication page, presents a login form to the user.
pub async fn auth() -> AuthTemplate {
    AuthTemplate
}

/// An error that can occur in a context where a HTML page is expected to be returned.
/// This is used to return a HTML page with a status code and the error message.
#[derive(Error, Debug)]
pub enum HtmlPageError {
    #[error("The given path `{0}` is invalid.")]
    InvalidPath(String),
    #[error("This resource could not be found.")]
    NotFound,
    #[error("An error occurred while querying the database.")]
    DatabaseError,
}

impl HtmlPageError {
    /// Converts this error into an appropriate HTTP status code.
    pub fn to_status_code(&self) -> StatusCode {
        match self {
            HtmlPageError::InvalidPath(_) => StatusCode::BAD_REQUEST,
            HtmlPageError::NotFound => StatusCode::NOT_FOUND,
            HtmlPageError::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for HtmlPageError {
    /// Converts this error into an axum HTTP response.
    fn into_response(self) -> Response<Body> {
        let template = ErrorTemplate {
            error: self.to_string(),
        };

        (self.to_status_code(), template).into_response()
    }
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/auth", get(auth))
        .route("/paste", get(paste::creation))
        .route("/paste/:slug", get(paste::page))
}

use serde::Serialize;

/// A generic error response for the API to return to clients.
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub message: String,
}

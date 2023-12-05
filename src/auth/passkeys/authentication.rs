use askama_axum::{
    IntoResponse,
    Response,
};
use axum::{
    http::StatusCode,
    Extension,
    Json,
};

use log::error;
use serde::{
    Deserialize,
    Serialize,
};
use sqlx::Postgres;
use thiserror::Error;
use tower_sessions::Session;
use uuid::Uuid;
use webauthn_rs::prelude::{
    Passkey,
    PasskeyAuthentication,
    PublicKeyCredential,
    RequestChallengeResponse,
    WebauthnError,
};

use crate::{
    auth::{
        passkeys::{
            backend::{
                AuthSession,
                BackendAuthParameters,
                PasskeyBackend,
            },
            PasskeyAuthState,
        },
        AuthParams,
    },
    db::{
        credentials::Credential,
        users::User,
    },
    http::{
        error::ApiError,
        ApiContext,
    },
};

/// A set of errors that can occur while registering a new user.
#[derive(Debug, Error)]
pub enum PasskeyAuthError {
    /// A user with the same email does not exist.
    #[error("A user with that name does not exist")]
    UserDoesNotExist,

    /// Could not create initial authentication challenge.
    #[error("Could not create initial authentication challenge: {0}")]
    ChallengeCreationFailure(WebauthnError),

    /// Something went wrong when trying to store authentication state in the session.
    #[error("Something went wrong when trying to store authentication state in the session: {0}")]
    SessionFailure(tower_sessions::session::Error),

    /// Authentication state was missing from the session.
    #[error(
    "Authentication state was missing from the session, are you sure you started authentication?"
    )]
    MissingSessionInfo,

    #[error("Something went wrong whilst verifying and completing authentication: {0}")]
    BackendAuthError(#[from] axum_login::Error<PasskeyBackend>),

    /// The backend checked the authentication challenge, but it was invalid.
    #[error("The backend checked the authentication challenge, but it was invalid")]
    BackendAuthInvalid,

    /// Could not log in user with auth backend.
    #[error("Could not log in user with auth backend: {0}")]
    AuthSessionFailure(axum_login::Error<PasskeyBackend>),

    /// An error occurred while communicating with the database.
    #[error("An error occurred while communicating with the database: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

impl IntoResponse for PasskeyAuthError {
    /// Converts the error into an [ApiError] and then a [Response] with an appropriate status code.
    fn into_response(self) -> Response {
        let status = match self {
            PasskeyAuthError::UserDoesNotExist => StatusCode::NOT_FOUND,
            PasskeyAuthError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            PasskeyAuthError::ChallengeCreationFailure(_) => StatusCode::INTERNAL_SERVER_ERROR,
            PasskeyAuthError::SessionFailure(_) => StatusCode::INTERNAL_SERVER_ERROR,
            PasskeyAuthError::AuthSessionFailure(_) => StatusCode::INTERNAL_SERVER_ERROR,
            PasskeyAuthError::MissingSessionInfo => StatusCode::BAD_REQUEST,
            PasskeyAuthError::BackendAuthError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            PasskeyAuthError::BackendAuthInvalid => StatusCode::BAD_REQUEST,
        };

        let error = ApiError {
            message: self.to_string(),
        };

        error!("{}", error.message);

        (status, Json(error)).into_response()
    }
}

/// A session that stores passkey authentication information across endpoints.
///
/// Not to be confused with [`AuthSession`], which is used to store the user's general
/// authentication state.
type AuthenticationSession = Session;

/// Registration info that is created in [start_register] and passed into [finish_register].
///
/// Passing is done via a [RegisterSession].
#[derive(Serialize, Deserialize)]
struct AuthenticationSessionInfo {
    email: String,
    user_unique_id: Uuid,
    auth_state: PasskeyAuthentication,
}

/// Starts the passkey authentication process for a new user.
///
/// This endpoint will create a new passkey authentication challenge. The generated
/// [RequestChallengeResponse] is passed back to the client and the resulting authentication state
/// will be passed to the [finish_authentication] endpoint via a [AuthenticationSession] to complete
/// the authentication when the client calls it.
pub async fn start_authentication(
    ctx: Extension<ApiContext>,
    Extension(state): Extension<PasskeyAuthState>,
    session: AuthenticationSession,
    Json(params): Json<AuthParams>,
) -> Result<Json<RequestChallengeResponse>, PasskeyAuthError> {
    // Get the user from the database if it exists.
    let user = sqlx::query_file_as!(User, "sql/get_user_by_username.sql", params.username)
        .fetch_optional(&ctx.db)
        .await?
        .map_or(Err(PasskeyAuthError::UserDoesNotExist), Ok)?;

    // language=postgresql
    let query = "SELECT * FROM credentials WHERE user_uuid = $1";
    let passkeys: Vec<Passkey> = sqlx::query_as::<Postgres, Credential>(query)
        .bind(user.uuid)
        .fetch_all(&ctx.db)
        .await?
        .iter()
        .map(|cred| cred.passkey.0.clone())
        .collect();

    let allow_credentials = passkeys.as_slice();
    let (rcr, auth_state) = state
        .webauthn
        .start_passkey_authentication(allow_credentials)
        .map_err(PasskeyAuthError::ChallengeCreationFailure)?;

    // Store necessary information in the session.
    let session_info = AuthenticationSessionInfo {
        email: params.username,
        user_unique_id: user.uuid,
        auth_state,
    };

    session
        .insert("auth_state", session_info)
        .map_err(PasskeyAuthError::SessionFailure)?;

    Ok(Json(rcr))
}

/// Finishes the passkey authentication process for a new user.
///
/// This endpoint will verify the [PublicKeyCredential] passed back by the client and validate it
/// with the authentication state that was stored in the [AuthenticationSession] by
/// [start_authentication].
///
/// If the authentication is successful, the user will be logged in.
pub async fn finish_authentication(
    Extension(state): Extension<PasskeyAuthState>,
    session: AuthenticationSession,
    mut auth_session: AuthSession,
    Json(public_key): Json<PublicKeyCredential>,
) -> Result<StatusCode, PasskeyAuthError> {
    // Get session info that should have been set in the start_register handler.
    // This can fail if the session info was never set, or if there was an error while
    // retrieving it.
    let session_info: AuthenticationSessionInfo = session
        .remove("auth_state")
        .map_err(PasskeyAuthError::SessionFailure)?
        .ok_or(PasskeyAuthError::MissingSessionInfo)?;

    let auth_params = BackendAuthParameters {
        auth_state: session_info.auth_state,
        challenge_response: public_key,
        user: None,
        user_uuid: session_info.user_unique_id,
        webauthn: state.webauthn,
    };

    let user = auth_session
        .authenticate(auth_params)
        .await
        .map_err(PasskeyAuthError::BackendAuthError)?
        .map_or(Err(PasskeyAuthError::BackendAuthInvalid), Ok)?;

    auth_session
        .login(&user)
        .await
        .map_err(PasskeyAuthError::AuthSessionFailure)?;

    Ok(StatusCode::OK)
}

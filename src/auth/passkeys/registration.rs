use axum::{
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
    Extension,
    Json,
};
use log::error;
use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;
use tower_sessions::Session;
use webauthn_rs::prelude::*;

use crate::{
    auth::{
        passkeys::{
            backend::{
                AuthSession,
                PasskeyBackend,
            },
            PasskeyAuthState,
        },
        AuthParams,
    },
    db::users::User,
    http::{
        error::ApiError,
        ApiContext,
    },
};

/// A set of errors that can occur while registering a new user.
#[derive(Debug, Error)]
pub enum PasskeyRegisterError {
    /// A user with the same name already exists.
    #[error("A user with that name already exists")]
    UserAlreadyExists,

    /// An error occurred while creating a new challenge.
    #[error("An error occurred while creating a new challenge: {0}")]
    ChallengeCreationFailure(WebauthnError),

    /// An error occurred while verifying and completing the registration.
    #[error("An error occurred while verifying and completing the registration: {0}")]
    RegistrationVerifyFailure(WebauthnError),

    /// An error occurred while storing registration state in the session.
    #[error("An error occurred while storing registration state in the session.")]
    SessionFailure(tower_sessions::session::Error),

    /// Registration state was missing from the session.
    #[error(
        "Registration state was missing from the session, are you sure you started registration?"
    )]
    MissingSessionInfo,

    /// An error occurred while logging in the user.
    #[error("An error occurred while logging in the user. {0}")]
    AuthSessionFailure(axum_login::Error<PasskeyBackend>),

    /// An error occurred while encoding the passkey to JSON.
    #[error("An error occurred while encoding the passkey to JSON.")]
    PasskeyJsonEncodeFailure(serde_json::Error),

    /// An error occurred while communicating with the database.
    #[error("An error occurred while communicating with the database.")]
    DatabaseError(#[from] sqlx::Error),
}

impl IntoResponse for PasskeyRegisterError {
    /// Converts the error into an [ApiError] and then a [Response] with an appropriate status code.
    fn into_response(self) -> Response {
        let status = match self {
            PasskeyRegisterError::UserAlreadyExists => StatusCode::CONFLICT,
            PasskeyRegisterError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            PasskeyRegisterError::ChallengeCreationFailure(_) => StatusCode::INTERNAL_SERVER_ERROR,
            PasskeyRegisterError::RegistrationVerifyFailure(_) => StatusCode::INTERNAL_SERVER_ERROR,
            PasskeyRegisterError::SessionFailure(_) => StatusCode::INTERNAL_SERVER_ERROR,
            PasskeyRegisterError::PasskeyJsonEncodeFailure(_) => StatusCode::INTERNAL_SERVER_ERROR,
            PasskeyRegisterError::AuthSessionFailure(_) => StatusCode::INTERNAL_SERVER_ERROR,
            PasskeyRegisterError::MissingSessionInfo => StatusCode::BAD_REQUEST,
        };

        let error = ApiError {
            message: self.to_string(),
        };

        error!("{}", error.message);

        (status, Json(error)).into_response()
    }
}

/// A session that stores passkey registration information across endpoints.
pub type RegisterSession = Session;

/// Registration info that is created in [start_register] and passed into [finish_register].
///
/// Passing is done via a [RegisterSession].
#[derive(Serialize, Deserialize)]
struct RegistrationSessionInfo {
    username: String,
    user_unique_id: Uuid,
    reg_state: PasskeyRegistration,
}

/// Starts the passkey registration process for a new user.
///
/// This endpoint will create a new passkey registration challenge. The generated
/// [CreationChallengeResponse] is passed back to the client and the resulting registration state
/// will be passed to the [finish_register] endpoint via a [RegisterSession] to complete the
/// registration when the client calls it.
pub async fn start_register(
    ctx: Extension<ApiContext>,
    Extension(state): Extension<PasskeyAuthState>,
    session: RegisterSession,
    Json(params): Json<AuthParams>,
) -> Result<impl IntoResponse, PasskeyRegisterError> {
    // Clear any previous registration state that may have been set.
    session.clear();

    let user_unique_id = Uuid::new_v4();

    // Make sure the user doesn't already exist.
    sqlx::query_file_as!(User, "sql/get_user_by_username.sql", params.username)
        .fetch_optional(&ctx.db)
        .await
        .map_err(PasskeyRegisterError::DatabaseError)?
        .map_or(Ok(()), |_| Err(PasskeyRegisterError::UserAlreadyExists))?;

    let (ccr, reg_state) = state
        .webauthn
        .start_passkey_registration(user_unique_id, &params.username, &params.username, None)
        .map_err(PasskeyRegisterError::ChallengeCreationFailure)?;

    // Construct the session info that will inevitably get passed to the finish_register handler.
    let session_info = RegistrationSessionInfo {
        username: params.username,
        user_unique_id,
        reg_state,
    };

    // Store the session info in the session.
    session
        .insert("reg_state", session_info)
        .map_err(PasskeyRegisterError::SessionFailure)?;

    Ok(Json(ccr))
}

/// Finishes the passkey registration process, creating a new user and associated credential.
///
/// This endpoint will verify the [RegisterPublicKeyCredential] passed back by the client and
/// validate it with the registration state that was stored in the [RegisterSession] by
/// [start_register].
///
/// If the registration is successful, a new user and credential will be created in the database
/// and the user will be automatically logged in.
pub async fn finish_register(
    ctx: Extension<ApiContext>,
    Extension(state): Extension<PasskeyAuthState>,
    session: RegisterSession,
    mut auth_session: AuthSession,
    Json(reg): Json<RegisterPublicKeyCredential>,
) -> Result<impl IntoResponse, PasskeyRegisterError> {
    // Get session info that should have been set in the start_register handler.
    // This can fail if the session info was never set, or if there was an error while
    // retrieving it.
    let session_info: RegistrationSessionInfo = session
        .remove("reg_state")
        .map_err(PasskeyRegisterError::SessionFailure)?
        .ok_or(PasskeyRegisterError::MissingSessionInfo)?;

    // Verify the registration and get the completed passkey.
    let passkey = state
        .webauthn
        .finish_passkey_registration(&reg, &session_info.reg_state)
        .map_err(PasskeyRegisterError::RegistrationVerifyFailure)?;

    // Time to insert the user into the database, we create a transaction to ensure that
    // the user and credential are inserted atomically.
    let mut tx = ctx
        .db
        .begin()
        .await
        .map_err(PasskeyRegisterError::DatabaseError)?;

    let user = sqlx::query_file_as!(
        User,
        "sql/insert_user.sql",
        session_info.username,
        session_info.user_unique_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(PasskeyRegisterError::DatabaseError)?;

    // Convert passkey to JSON and insert it into the database.
    let passkey =
        serde_json::to_value(passkey).map_err(PasskeyRegisterError::PasskeyJsonEncodeFailure)?;
    sqlx::query_file_as!(Credential, "sql/insert_credential.sql", user.uuid, passkey)
        .execute(&mut *tx)
        .await
        .map_err(PasskeyRegisterError::DatabaseError)?;

    tx.commit()
        .await
        .map_err(PasskeyRegisterError::DatabaseError)?;

    // Automatically log the user in.
    auth_session
        .login(&user)
        .await
        .map_err(PasskeyRegisterError::AuthSessionFailure)?;

    Ok(StatusCode::OK)
}

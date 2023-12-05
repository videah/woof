use std::sync::Arc;

use async_trait::async_trait;
use axum_login::{
    AuthUser,
    AuthnBackend,
    UserId,
};

use sqlx::{
    PgPool,
    Postgres,
};
use thiserror::Error;
use webauthn_rs::prelude::*;

use crate::db::{
    credentials::Credential,
    users::User,
};

impl AuthUser for User {
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.username.as_bytes()
    }
}

/// Parameters to pass to [`PasskeyBackend::authenticate`].
#[derive(Debug, Clone)]
pub struct BackendAuthParameters {
    /// An in-progress passkey authentication state.
    pub auth_state: PasskeyAuthentication,
    /// The response to a passkey challenge that was passed to the server.
    pub challenge_response: PublicKeyCredential,
    /// The user that is attempting to authenticate.
    ///
    /// It's possible we don't know who this is yet, this can happen when an autofill requests
    /// a passkey challenge before the user has entered an email.
    pub user: Option<User>,
    /// The UUID of the user that is attempting to authenticate.
    pub user_uuid: Uuid,
    /// The webauthn instance to use for authentication.
    pub webauthn: Arc<Webauthn>,
}

/// A backend for authenticating users with passkeys and a PostgreSQL database.
#[derive(Debug, Clone)]
pub struct PasskeyBackend {
    db: PgPool,
}

impl PasskeyBackend {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

/// Errors that can occur during passkey authentication whilst operating in [`PasskeyBackend`].
#[derive(Error, Debug)]
pub enum BackendAuthError {
    /// Could not update passkey credential's counter.
    #[error("Could not update passkey credential's counter: {0}")]
    CredentialUpdateFailure(sqlx::Error),

    /// Could not grab a stored credential after validating the authentication.
    #[error("Could not grab a stored credential after validating the authentication.")]
    StoredCredentialFailure(sqlx::Error),

    /// The credential is orphaned and has no user associated with it.
    #[error("The credential is orphaned and has no user associated with it.")]
    OrphanedCredential(sqlx::Error),

    /// The credential counter is out of sync with the stored value.
    ///
    /// This is a potential sign of a cloned credential.
    #[error("The credential counter is out of sync with the stored value.")]
    CounterDiscrepancy,

    /// Database error.
    #[error("Database error")]
    DatabaseFailure(#[from] sqlx::Error),
}

#[async_trait]
impl AuthnBackend for PasskeyBackend {
    type User = User;
    type Credentials = BackendAuthParameters;
    type Error = BackendAuthError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        // Attempt to complete the authentication process by validating the challenge response.
        let auth_result = creds
            .webauthn
            .finish_passkey_authentication(&creds.challenge_response, &creds.auth_state)
            .ok();

        // Unpack our authentication result and return early if we don't have one, since that means
        // the authentication failed.
        let auth_result = match auth_result {
            Some(result) => result,
            None => return Ok(None),
        };

        if auth_result.needs_update() {
            // Update the credential counter if needed.
            // Unlikely to be necessary since most passkeys don't even have a mechanism for holding
            // their count, but should be handled regardless just in case 🤞
            self.update_credential_counter(creds.user_uuid, &auth_result)
                .await?;
        }

        // It's possible that we don't know the user we're meant to be authenticating yet like in
        // the case of an autofill where the challenge is given to the client before we are ever
        // passed a username.
        //
        // In this case we need to look up the user from the provided credential.
        let id = auth_result.cred_id();
        let user = self.get_user_from_credentials(creds.user, id).await?;
        Ok(user)
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
            .fetch_optional(&self.db)
            .await?;

        Ok(user)
    }
}

impl PasskeyBackend {
    /// Increment the counter of a successfully authenticated credential and update the database.
    async fn update_credential_counter(
        &self,
        user_uuid: Uuid,
        auth_result: &AuthenticationResult,
    ) -> Result<(), BackendAuthError> {
        let mut stored_creds =
            sqlx::query_as::<Postgres, Credential>("SELECT * FROM credentials WHERE user_id = $1")
                .bind(user_uuid)
                .fetch_all(&self.db)
                .await
                .map_err(BackendAuthError::StoredCredentialFailure)?;

        //TODO(videah): check counter discrepancies to detect cloning.

        for cred in stored_creds.iter_mut() {
            let is_valid_credential = cred.passkey.update_credential(auth_result);
            if let Some(updated) = is_valid_credential {
                if updated {
                    sqlx::query("UPDATE credentials SET passkey = $1 WHERE id = $2")
                        .bind(cred.passkey.clone())
                        .bind(cred.id)
                        .execute(&self.db)
                        .await
                        .map_err(BackendAuthError::CredentialUpdateFailure)?;
                    break;
                }
            }
        }

        Ok(())
    }

    /// Get a user assigned to a credential ID.
    ///
    /// If the user is already known, it will be returned. Otherwise, a user will be looked up
    /// from the database.
    ///
    /// If the credential is orphaned and has no user assigned to it an error will be returned.
    async fn get_user_from_credentials(
        &self,
        potential_user: Option<User>,
        cred_id: &CredentialID,
    ) -> Result<Option<User>, BackendAuthError> {
        let user = match potential_user {
            Some(user) => Some(user),
            None => {
                // language=postgresql
                let query =
                    "SELECT * FROM credentials WHERE passkey::json->'cred'->>'cred_id' = $1";
                let cred = sqlx::query_as::<Postgres, Credential>(query)
                    .bind(cred_id.to_string())
                    .fetch_one(&self.db)
                    .await
                    .map_err(BackendAuthError::StoredCredentialFailure)?;

                let user = sqlx::query_as::<Postgres, User>("SELECT * FROM users WHERE uuid = $1")
                    .bind(cred.user_uuid)
                    .fetch_one(&self.db)
                    .await
                    .map_err(BackendAuthError::OrphanedCredential)?;

                Some(user)
            }
        };

        Ok(user)
    }
}

/// A convenience type alias for [`axum_login::AuthSession`] with our concrete backend.
///
/// Not to be confused with [RegisterSession] or [AuthenticateSession] which are used for
/// passing data between passkey register and auth handlers.
pub type AuthSession = axum_login::AuthSession<PasskeyBackend>;

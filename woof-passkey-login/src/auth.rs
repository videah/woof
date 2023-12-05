//! This module contains all the logic for the WebAuthn authentication flow.

use gloo_net::http::Request;
use seed::{
    prelude::*,
    *,
};
use serde::{
    de::DeserializeOwned,
    Deserialize,
    Serialize,
};
use thiserror::Error;
use webauthn_rs_proto::{
    CreationChallengeResponse,
    PublicKeyCredential,
    RegisterPublicKeyCredential,
    RequestChallengeResponse,
};

use crate::{
    views::ViewState,
    Msg,
};

/// Holds all the state for the authentication component.
pub struct AuthModel {
    /// The current value of the username input box.
    pub input_value: String,
    /// The last username that was used to start the registration/authentication process.
    pub last_username: String,
    /// The current state of the view.
    pub view_state: ViewState,
}

/// Parameters sent to the server to start the registration/authentication process.
#[derive(Serialize)]
pub struct AuthServerParams {
    pub username: String,
}

impl AuthModel {
    /// Start the registration process for a user.
    ///
    /// This sends the user's username to the server, which will respond with a
    /// [CreationChallengeResponse] if successful. This should then passed to
    /// [sign_register_challenge].
    ///
    /// If the server responds with an error, it will be displayed to the user.
    pub fn start_register(&mut self, username: String, orders: &mut impl Orders<Msg>) {
        self.last_username = username.clone();
        orders.perform_cmd(async move {
            match get_challenge("/api/users/start_register", username.clone()).await {
                Ok(ccr) => Msg::SignRegisterChallenge(ccr),
                Err(err) => Msg::Error(err.to_string()),
            }
        });
    }

    /// Initiate's the browser's passkey registration process, using the [CreationChallengeResponse]
    /// from the server.
    ///
    /// This will prompt the user to register a new passkey and then pass the result to
    /// [finish_register]. If the user cancels the registration process, an error will be displayed.
    ///
    /// After the user has registered a passkey, [finish_register] should be called with the
    /// [RegisterPublicKeyCredential] passed to it.
    pub fn sign_register_challenge(
        &mut self,
        ccr: CreationChallengeResponse,
        orders: &mut impl Orders<Msg>,
    ) {
        // First, convert from our webauthn proto json safe format, into the browser
        // compatible struct, with everything decoded as needed.
        let c_options: web_sys::CredentialCreationOptions = ccr.into();

        // Now, we can create a promise using the browser's credential creation API.
        let promise = window()
            .navigator()
            .credentials()
            .create_with_options(&c_options)
            .expect_throw("Could not create credential");

        // We need to convert the promise into a future so we can await it.
        let signing_future = JsFuture::from(promise);
        orders.perform_cmd(async move {
            // Await the promise, if it resolves, we have a PublicKeyCredential, if it rejects,
            // the user *probably* cancelled the registration process. It's possible that the
            // browser could reject for other reasons, but we'll just assume it's a cancellation for
            // now.
            let jsval = match signing_future.await {
                Ok(val) => val,
                Err(_) => {
                    return Msg::Error("Authentication cancelled".to_string());
                }
            };

            // Convert from the raw js value into the expected PublicKeyCredential
            let w_rpkc = web_sys::PublicKeyCredential::from(jsval);

            // Serialise the web_sys::pkc into the webauthn proto version, ready to
            // handle/transmit.
            let rpkc = RegisterPublicKeyCredential::from(w_rpkc);

            Msg::FinishRegister(rpkc)
        });
    }

    /// Finish the registration process by sending the [RegisterPublicKeyCredential] to the server.
    /// This completes the registration process and the user can now login with their passkey.
    ///
    /// If the server responds with an error it will be displayed to the user.
    pub fn finish_register(
        &mut self,
        rpkc: RegisterPublicKeyCredential,
        orders: &mut impl Orders<Msg>,
    ) {
        orders.perform_cmd(async move {
            match submit_credential("/api/users/finish_registration", rpkc).await {
                Ok(_) => Msg::Success,
                Err(err) => Msg::Error(err.to_string()),
            }
        });
    }

    /// Start the authentication process for a user.
    ///
    /// This sends the user's username to the server, which will respond with a
    /// [RequestChallengeResponse] if successful. This should then passed to
    /// [sign_authentication_challenge].
    ///
    /// If the server responds with an error, it will be displayed to the user.
    pub fn start_authentication(&mut self, username: String, orders: &mut impl Orders<Msg>) {
        self.last_username = username.clone();
        orders.perform_cmd(async move {
            match get_challenge("/api/users/start_authentication", username.clone()).await {
                Ok(rcr) => Msg::SignAuthenticationChallenge(rcr),
                Err(err) => Msg::Error(err.to_string()),
            }
        });
    }

    /// Initiate's the browser's passkey authentication process, using the
    /// [RequestChallengeResponse] from the server.
    ///
    /// This will prompt the user to authenticate with their passkey. If the user cancels the
    /// authentication process an error will be displayed.
    ///
    /// After the user has registered a passkey, [finish_register] should be called with the
    /// [PublicKeyCredential] passed to it.
    pub fn sign_authentication_challenge(
        &mut self,
        rcr: RequestChallengeResponse,
        orders: &mut impl Orders<Msg>,
    ) {
        // First, convert from our webauthn proto json safe format, into the browser
        // compatible struct, with everything decoded as needed.
        let c_options: web_sys::CredentialRequestOptions = rcr.into();

        // Now, we can create a promise using the browser's credential API.
        let promise = window()
            .navigator()
            .credentials()
            .get_with_options(&c_options)
            .expect_throw("Could not create credential");

        // We need to convert the promise into a future so we can await it.
        let signing_future = JsFuture::from(promise);
        orders.perform_cmd(async move {
            // Await the promise, if it resolves, we have a PublicKeyCredential, if it rejects,
            // the user *probably* cancelled the registration process. It's possible that the
            // browser could reject for other reasons, but we'll just assume it's a cancellation for
            // now.
            let jsval = match signing_future.await {
                Ok(val) => val,
                Err(_) => {
                    return Msg::Error("Authentication cancelled".to_string());
                }
            };

            // Convert from the raw js value into the expected PublicKeyCredential
            let w_pkc = web_sys::PublicKeyCredential::from(jsval);

            // Serialise the web_sys::pkc into the webauthn proto version, ready to
            // handle/transmit.
            let pkc = PublicKeyCredential::from(w_pkc);

            Msg::FinishAuthentication(pkc)
        });
    }

    /// Finish the authentication process by sending the [PublicKeyCredential] to the server.
    /// This completes the authentication process and the user is now logged in with their passkey.
    ///
    /// If the server responds with an error it will be displayed to the user.
    pub fn finish_authentication(
        &mut self,
        pkc: PublicKeyCredential,
        orders: &mut impl Orders<Msg>,
    ) {
        orders.perform_cmd(async move {
            match submit_credential("/api/users/finish_authentication", pkc).await {
                Ok(_) => Msg::Success,
                Err(err) => Msg::Error(err.to_string()),
            }
        });
    }
}

/// An error returned by the server API.
#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub message: String,
}

/// An error that can occur during the authentication process.
#[derive(Debug, Error)]
pub enum AuthProcessError {
    /// Could not fetch the challenge from the server API.
    #[error("Could not fetch challenge from server API: {0}")]
    FetchChallengeFailure(gloo_net::Error),

    /// Could not parse the challenge response from the server.
    #[error("Could not parse challenge response: {0}")]
    ChallengeParseFailure(gloo_net::Error),

    /// The server responded with a non-200 status code and a possible error message.
    #[error("{0}")]
    ApiError(String),

    /// The server responded with a non-200 status code but an error message could not be parsed.
    #[error("An error occurred but could not be parsed.")]
    ApiErrorParseFailure(gloo_net::Error),
}

/// Send a request to the server to get a passkey challenge.
///
/// This will return a [AuthProcessError] if the request fails, or the server responds with an
/// error.
///
/// This is intended to be used for both registration and authentication challenges, so the
/// response type is generic. In this case the response type [T] should be either
/// [CreationChallengeResponse] or [RequestChallengeResponse].
pub async fn get_challenge<T>(endpoint: &str, email: String) -> Result<T, AuthProcessError>
where
    T: DeserializeOwned,
{
    let params = AuthServerParams { username: email };
    let request = Request::post(endpoint)
        .header("Content-Type", "application/json")
        .json(&params)
        .map_err(AuthProcessError::FetchChallengeFailure)?;

    let response = request
        .send()
        .await
        .map_err(AuthProcessError::FetchChallengeFailure)?;

    // If the response is not 200, we have an error and throw whatever the server gave us back.
    if response.status() != 200 {
        let error: ApiError = response
            .json()
            .await
            .map_err(AuthProcessError::ApiErrorParseFailure)?;

        return Err(AuthProcessError::ApiError(error.message));
    }

    let challenge_response: T = response
        .json()
        .await
        .map_err(AuthProcessError::ChallengeParseFailure)?;

    Ok(challenge_response)
}

/// Send a credential to the server to complete the registration/authentication process.
///
/// This will return a [AuthProcessError] if the request fails, or the server responds with an
/// error.
///
/// This is intended to be used for both registration and authentication challenges, so the
/// response type is generic. In this case the response type [T] should be either
/// [RegisterPublicKeyCredential] or [PublicKeyCredential].
pub async fn submit_credential<T>(endpoint: &str, credential: T) -> Result<(), AuthProcessError>
where
    T: Serialize,
{
    let request = Request::post(endpoint)
        .header("Content-Type", "application/json")
        .json(&credential)
        .map_err(AuthProcessError::FetchChallengeFailure)?;

    request
        .send()
        .await
        .map_err(AuthProcessError::FetchChallengeFailure)?;

    Ok(())
}

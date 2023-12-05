//! WebAuthn passkey authentication component intended to be used with Woof.

pub mod auth;
pub mod svg;
pub mod utils;
pub mod views;

use gloo_timers::callback::Timeout;
use seed::{
    prelude::*,
    *,
};
use web_sys::HtmlElement;
use webauthn_rs_proto::{
    CreationChallengeResponse,
    PublicKeyCredential,
    RegisterPublicKeyCredential,
    RequestChallengeResponse,
};

use crate::{
    auth::AuthModel,
    utils::set_panic_hook,
    views::ViewState,
};

/// Initializes the application model.
pub fn init(_: Url, _: &mut impl Orders<Msg>) -> AuthModel {
    AuthModel {
        view_state: ViewState::Init,
        last_username: String::new(),
        input_value: String::new(),
    }
}

/// Messages used to communicate and process state changes across the application.
/// These are handled by the [update] function.
pub enum Msg {
    /// Sent when the input value of the username text box changes.
    InputChanged(String),

    /// Sent when the user presses the register button.
    BeginRegister,

    /// Sent when the registration has started and the server has sent a challenge.
    /// Starts the process of signing the challenge with the browser credential API.
    ///
    /// Holds the [CreationChallengeResponse] received from the server.
    SignRegisterChallenge(CreationChallengeResponse),

    /// Sent when the a credential is successfully created by the browser.
    /// Sends the signed challenge to the server to finish the registration process.
    ///
    /// Holds the [RegisterPublicKeyCredential] received from the browser.
    FinishRegister(RegisterPublicKeyCredential),

    /// Sent when the user presses the login button.
    BeginAuthentication,
    /// Sent when the authentication has started and the server has sent a challenge.
    /// Starts the process of signing the challenge with the browser credential API.
    ///
    /// Holds the [RequestChallengeResponse] received from the server.
    SignAuthenticationChallenge(RequestChallengeResponse),

    /// Sent when the a credential is successfully created by the browser.
    /// Sends the signed challenge to the server to finish the authentication process.
    ///
    /// Holds the [PublicKeyCredential] received from the browser.
    FinishAuthentication(PublicKeyCredential),

    /// Sent when the authentication/registration process is successful.
    Success,

    /// Sent when an error occurs during the authentication/registration process.
    ///
    /// Holds the error message string.
    Error(String),

    /// A no-op message used to satisfy the compiler. This is used in the [input_ev] and
    /// [keyboard_ev] functions in the authentication view and ultimately does nothing.
    NoOp,
}

/// Updates the model based on the message received.
/// This is where the bulk of the application logic is handled.
pub fn update(msg: Msg, model: &mut AuthModel, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::InputChanged(text) => model.input_value = text,
        Msg::Error(err) => model.view_state = ViewState::Error(err),
        // Registration
        Msg::BeginRegister => {
            // If the input value is empty, don't do anything.
            // Just a basic check to prevent empty usernames
            if !model.input_value.is_empty() {
                model.view_state = ViewState::Waiting;
                model.start_register(model.input_value.clone(), orders);
            } else {
                model.view_state = ViewState::Error("Username cannot be empty".to_string());
            }
        }
        Msg::SignRegisterChallenge(challenge_response) => {
            model.sign_register_challenge(challenge_response, orders);
        }
        Msg::FinishRegister(register_response) => {
            model.finish_register(register_response, orders);
        }
        // Authentication
        Msg::BeginAuthentication => {
            if !model.input_value.is_empty() {
                model.view_state = ViewState::Waiting;
                model.start_authentication(model.input_value.clone(), orders);
            } else {
                model.view_state = ViewState::Error("Username cannot be empty".to_string());
            }
        }
        Msg::SignAuthenticationChallenge(challenge_response) => {
            model.sign_authentication_challenge(challenge_response, orders);
        }
        Msg::FinishAuthentication(authentication_response) => {
            model.finish_authentication(authentication_response, orders)
        }
        Msg::Success => {
            // Update the view state to success, displaying the last username that was stored
            // right before the authentication/registration process started.
            model.view_state = ViewState::Success(model.last_username.clone());

            // Replace any instances of the class `card` with `card-success`.
            if let Some(element) = document().get_element_by_id("auth-card") {
                let html_element: HtmlElement = element.unchecked_into();
                html_element.set_class_name("card-success");
            }

            // Wait a little bit before redirecting to the desired page. This gives the user
            // enough time to see the success message.
            Timeout::new(1500, move || {
                // Access the current URL and get the query parameters.
                let url = Url::current();
                let query_params = url.search();

                // Get the value of the redirect query parameter, or use the index route if it
                // doesn't exist.
                let default_redirect = "/".to_string();
                let redirect_path = match query_params.get("redirect") {
                    Some(redirect) => redirect.first().unwrap_or(&default_redirect),
                    None => &default_redirect,
                };

                // Redirect to the next page which can be specified by the `redirect` query
                // parameter. Otherwise we redirect to the index route.
                let redirect_path_segments: Vec<&str> = redirect_path.split('/').collect();
                let redirect = Url::current().set_path(redirect_path_segments);
                let redirect = redirect.set_search(UrlSearch::default());
                redirect.go_and_load();
            })
            .forget();
        }
        Msg::NoOp => {}
    }
}

/// Renders the view based on the current state of the application.
pub fn view(model: &AuthModel) -> Node<Msg> {
    match model.view_state {
        ViewState::Error(ref err) => views::view(&model.view_state, Some(err)),
        _ => views::view(&model.view_state, None),
    }
}

/// Bind and render the application to the element with the id `app`.
#[wasm_bindgen(start)]
pub fn start() {
    set_panic_hook();
    App::start("app", init, update, view);
}

//! Defines the different HTML views for various states of the component.

use seed::{
    prelude::*,
    *,
};

use crate::{
    svg::{
        passkey_icon,
        profile_icon,
        spinner_icon,
        success_icon,
        warning_icon,
    },
    Msg,
};

/// Defines the different states the authentication view can be in.
/// This is used to determine which view to render independently of the model message.
#[derive(PartialEq)]
pub enum ViewState {
    /// The initial state of the view.
    Init,
    /// The view is waiting for a response from the user/server.
    Waiting,
    /// The view has successfully authenticated the user, holds the now authenticated username.
    Success(String),
    /// The view has encountered an error, holds the error message.
    Error(String),
}

/// Defines the HTML view for the authentication component and reacts to changes in [ViewState].
///
/// An error message is displayed if [ViewState] is [ViewState::Error] and the error text is not
/// None.
pub fn view(state: &ViewState, error_text: Option<&String>) -> Node<Msg> {
    match state {
        ViewState::Success(user) => view_success(user),
        _ => {
            div![
                div![
                    C!["flex", "flex-col", "gap", "items-center", "justify-center",],
                    div![
                        C!["input-container"],
                        i![C!["input-icon"], profile_icon()],
                        input![
                            C!["input-purple"],
                            attrs! {
                                At::Placeholder => "Enter your username",
                            },
                            // We store the input value in the model by sending a message every
                            // time the input changes.
                            input_ev(Ev::Input, Msg::InputChanged),
                            // We also send a message when the user presses enter to start the
                            // authentication process.
                            keyboard_ev(Ev::KeyDown, |keyboard_event| {
                                if keyboard_event.key() == "Enter" {
                                    Msg::BeginAuthentication
                                } else {
                                    Msg::NoOp
                                }
                            })
                        ],
                    ],
                    button![
                        C!["button-purple"],
                        attrs! {
                            At::Disabled => (state == &ViewState::Waiting).as_at_value(),
                        },
                        ev(Ev::Click, |_| Msg::BeginAuthentication),
                        passkey_icon(),
                        span!["Sign in with ", strong!("Passkey")],
                    ],
                    button![
                        C!["button-gray"],
                        attrs! {
                            At::Disabled => (state == &ViewState::Waiting).as_at_value(),
                        },
                        ev(Ev::Click, |_| Msg::BeginRegister),
                        "Register"
                    ],
                ],
                IF!(state != &ViewState::Waiting => error_message(error_text)),
                IF!(state == &ViewState::Waiting => waiting_message()),
                div![
                    C!["flex", "flex-row", "justify-between pt-4"],
                    a![
                        C![
                            "text-sm",
                            "text-gray-500",
                            "hover:text-gray-700",
                            "underline"
                        ],
                        "What is a passkey?"
                    ],
                    a![
                        C![
                            "text-sm",
                            "text-gray-500",
                            "hover:text-gray-700",
                            "underline"
                        ],
                        "Upload Anonymously"
                    ]
                ]
            ]
        }
    }
}

/// Defines the HTML view for the error message.
/// If `error_text` is None, the error message is made invisible.
pub fn error_message(error_text: Option<&String>) -> Node<Msg> {
    div![
        C![
            "text-red-500 w-full fade-in mt-2",
            IF!(error_text.is_none() => "invisible"),
        ],
        div![
            C!["flex flex-row items-center"],
            warning_icon(),
            span![error_text.unwrap_or(&"Nothing yet...".to_string())]
        ]
    ]
}

/// Defines the HTML view for the waiting message.
/// This looks like the error message, but with a spinner instead of a warning icon.
pub fn waiting_message() -> Node<Msg> {
    div![
        C!["text-gray-500 w-full fade-in mt-2 fade-in"],
        div![
            C!["flex flex-row items-center"],
            spinner_icon(),
            span!["Waiting for authentication..."]
        ]
    ]
}

/// Defines the HTML view for the success message.
/// This plays a fade-in animation and displays the user's name.
pub fn view_success(user: &String) -> Node<Msg> {
    div![
        C!["flex", "flex-row", "items-center", "fade-in-up"],
        success_icon(),
        div![
            C!["flex", "flex-col", "test"],
            span!["Welcome"],
            span!(strong![C!["text-4xl"], format!("{}", user)])
        ]
    ]
}

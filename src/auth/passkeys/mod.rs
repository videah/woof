use std::sync::Arc;

use webauthn_rs::{
    prelude::Url,
    Webauthn,
    WebauthnBuilder,
};

pub mod authentication;
pub mod backend;
pub mod registration;

/// Configuration for the Webauthn instance used for passkey authentication.
#[derive(Clone)]
pub struct PasskeyAuthState {
    /// The Webauthn instance used for passkey authentication.
    pub webauthn: Arc<Webauthn>,
    /// The hostname of the server (i.e `videah.net`) used as the RP ID and origin.
    pub hostname: String,
    /// The appid required for communicating with Apple devices.
    pub appid: String,
}

impl PasskeyAuthState {
    pub fn new(rp_id: String, appid: String) -> PasskeyAuthState {
        let rp_origin = Url::parse(&format!("https://{rp_id}")).unwrap();
        let builder = WebauthnBuilder::new(&rp_id, &rp_origin).unwrap();
        let builder = builder.rp_name("woof");

        let webauthn = Arc::new(builder.build().unwrap());
        PasskeyAuthState {
            webauthn,
            hostname: rp_id,
            appid,
        }
    }
}

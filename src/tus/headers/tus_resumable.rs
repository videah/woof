use std::fmt::Display;

use axum::http::HeaderName;
use headers::Header;
use http::StatusCode;

use crate::tus::headers::{
    tus_version::TusVersionHeader,
    Version,
};

static CUSTOM_HEADER: &'static str = "tus-resumable";
static HEADER_NAME: HeaderName = HeaderName::from_static(CUSTOM_HEADER);

/// # Tus-Resumable
/// The [TusResumableHeader] header MUST be included in every request and response except for
/// OPTIONS requests. The value MUST be the version of the protocol used by the Client or the
/// Server.
///
/// If the version specified by the Client is not supported by the Server, it MUST respond with the
/// [StatusCode::PRECONDITION_FAILED] status and MUST include the [TusVersionHeader] header into the
/// response. In addition, the Server MUST NOT process the request.
pub struct TusResumableHeader(Version);

impl Header for TusResumableHeader {
    fn name() -> &'static HeaderName {
        &HEADER_NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i http::HeaderValue>,
    {
        let value = values
            .next()
            .ok_or_else(headers::Error::invalid)?
            .to_str()
            .map_err(|_| headers::Error::invalid())?;

        let version = Version::new(value).map_err(|_| headers::Error::invalid())?;

        Ok(TusResumableHeader(version))
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<http::HeaderValue>,
    {
        let value = http::HeaderValue::from_str(&self.0.to_string()).unwrap();
        values.extend(std::iter::once(value));
    }
}

use std::fmt::Display;

use axum::http::HeaderName;
use headers::Header;

use crate::tus::extensions::Extension;

static CUSTOM_HEADER: &'static str = "tus-extension";
static HEADER_NAME: HeaderName = HeaderName::from_static(CUSTOM_HEADER);

/// # Tus-Extension
/// The [TusExtensionHeader] response header MUST be a comma-separated list of the extensions
/// supported by the Server. If no extensions are supported, the [TusExtensionHeader] header MUST be
/// omitted.
pub struct TusExtensionHeader(Vec<Extension>);

impl Header for TusExtensionHeader {
    fn name() -> &'static HeaderName {
        &HEADER_NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i http::HeaderValue>,
    {
        let mut extensions = Vec::new();

        for value in values {
            let value_str = value.to_str().map_err(|_| headers::Error::invalid())?;
            let extension_strings = value_str.split(',');

            for ext in extension_strings {
                let extension = match ext.trim() {
                    "creation" => Extension::Creation,
                    _ => return Err(headers::Error::invalid()),
                };
                extensions.push(extension);
            }
        }

        Ok(TusExtensionHeader(extensions))
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<http::HeaderValue>,
    {
        let extension_strings: Vec<String> = self.0.iter().map(Extension::to_string).collect();
        let combined = extension_strings.join(",");
        if let Ok(header_value) = http::HeaderValue::from_str(&combined) {
            values.extend(std::iter::once(header_value));
        }
    }
}

use axum::http::HeaderName;
use headers::Header;

static CUSTOM_HEADER: &'static str = "upload-length";
static HEADER_NAME: HeaderName = HeaderName::from_static(CUSTOM_HEADER);

/// # Upload-Length
/// The [UploadLengthHeader] request and response header indicates the size of the entire upload in
/// bytes. The value MUST be a non-negative integer.
pub struct UploadLengthHeader(u64);

impl Header for UploadLengthHeader {
    fn name() -> &'static HeaderName {
        &HEADER_NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i http::HeaderValue>,
    {
        values
            .next()
            .and_then(|value| value.to_str().ok()?.parse().ok())
            .map(UploadLengthHeader)
            .ok_or_else(headers::Error::invalid)
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<http::HeaderValue>,
    {
        let value = http::HeaderValue::from_str(&self.0.to_string()).unwrap();
        values.extend(std::iter::once(value));
    }
}

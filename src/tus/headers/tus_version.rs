use std::fmt::Display;

use axum::http::HeaderName;
use headers::Header;

use crate::tus::headers::Version;

static CUSTOM_HEADER: &'static str = "tus-version";
static HEADER_NAME: HeaderName = HeaderName::from_static(CUSTOM_HEADER);

/// # Tus-Version
/// The [TusVersionHeader] response header MUST be a comma-separated list of protocol versions
/// supported by the Server. The list MUST be sorted by Server’s preference where the first one is
/// the most preferred one.
pub struct TusVersionHeader(Vec<Version>);

impl Header for TusVersionHeader {
    fn name() -> &'static HeaderName {
        &HEADER_NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i http::HeaderValue>,
    {
        let mut versions = Vec::new();

        for value in values {
            let value_str = value.to_str().map_err(|_| headers::Error::invalid())?;
            let version_strings = value_str.split(',');

            for version_str in version_strings {
                let version =
                    Version::new(version_str.trim()).map_err(|_| headers::Error::invalid())?;
                versions.push(version);
            }
        }

        Ok(TusVersionHeader(versions))
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<http::HeaderValue>,
    {
        let version_strings: Vec<String> = self.0.iter().map(Version::to_string).collect();
        let combined = version_strings.join(",");
        if let Ok(header_value) = http::HeaderValue::from_str(&combined) {
            values.extend(std::iter::once(header_value));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_with_valid_header() {
        let value = http::HeaderValue::from_static("1.2.3,4.5.6");
        let mut values = vec![&value].into_iter();
        let tus_version = TusVersionHeader::decode(&mut values);
        assert!(tus_version.is_ok());
        let tus_version = tus_version.unwrap();
        assert_eq!(tus_version.0.len(), 2);
        assert_eq!(tus_version.0[0].to_string(), "1.2.3");
        assert_eq!(tus_version.0[1].to_string(), "4.5.6");
    }

    #[test]
    fn decode_with_invalid_header() {
        let value = http::HeaderValue::from_static("1.2,4.5.6");
        let mut values = vec![&value].into_iter();
        let tus_version = TusVersionHeader::decode(&mut values);
        assert!(tus_version.is_err());
    }

    #[test]
    fn version_encode() {
        let tus_version = TusVersionHeader(vec![
            Version::new("1.0.0").unwrap(),
            Version::new("0.2.2").unwrap(),
            Version::new("0.2.1").unwrap(),
        ]);
        let mut values = Vec::new();
        tus_version.encode(&mut values);
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].to_str().unwrap(), "1.0.0,0.2.2,0.2.1");
    }
}

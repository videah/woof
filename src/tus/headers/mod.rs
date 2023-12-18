use std::fmt::Display;

pub mod tus_extension;
pub mod tus_resumable;
pub mod tus_version;
pub mod upload_length;
pub mod upload_offset;

pub use crate::tus::headers::{
    tus_extension::TusExtensionHeader,
    tus_resumable::TusResumableHeader,
    tus_version::TusVersionHeader,
    upload_length::UploadLengthHeader,
    upload_offset::UploadOffsetHeader,
};

#[derive(Debug, PartialEq)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl Version {
    pub fn new(s: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = s.split('.').collect();

        if parts.len() == 3 {
            let major = parts[0]
                .parse::<u32>()
                .map_err(|_| "Invalid major version")?;
            let minor = parts[1]
                .parse::<u32>()
                .map_err(|_| "Invalid minor version")?;
            let patch = parts[2]
                .parse::<u32>()
                .map_err(|_| "Invalid patch version")?;

            Ok(Version {
                major,
                minor,
                patch,
            })
        } else {
            Err("Invalid version string format")
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format!("{}.{}.{}", self.major, self.minor, self.patch)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_creation_with_valid_string() {
        let version = Version::new("1.2.3");
        assert!(version.is_ok());
        let version = version.unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn version_creation_with_invalid_string() {
        let version = Version::new("1.2");
        assert!(version.is_err());
    }

    #[test]
    fn version_display_format() {
        let version = Version::new("1.2.3").unwrap();
        assert_eq!(version.to_string(), "1.2.3");
    }
}

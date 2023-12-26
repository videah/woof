use serde::{
    Deserialize,
    Serialize,
};
use sqlx::{
    postgres::{
        PgTypeInfo,
        PgValueRef,
    },
    types::time::OffsetDateTime,
    Decode,
    FromRow,
    Postgres,
    Row,
};
use thiserror::Error;

/// A slug string, consisting of 4 words separated by dashes. (e.g. `this-is-a-slug`)
/// This is used to identify a resource like a paste or a file.
///
/// This type implements [`Decode`] for decoding values from the database, strictly checking and
/// enforcing the format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlugString(String);

#[derive(Error, Debug)]
pub enum SlugError {
    #[error("Invalid slug format, expected 4 words separated by dashes, got: {0}")]
    InvalidFormat(String),
}

impl SlugString {
    /// Creates a new slug from the given string.
    pub fn new(input: &str) -> Result<SlugString, SlugError> {
        if SlugString::is_valid(input) {
            Ok(SlugString(input.to_string()))
        } else {
            Err(SlugError::InvalidFormat(input.to_string()))
        }
    }

    /// Checks if the given string is a valid slug.
    /// A valid slug is 4 words separated by dashes (e.g. `this-is-a-slug`).
    pub fn is_valid(input: &str) -> bool {
        let parts: Vec<&str> = input.split('-').collect();
        parts.len() == 4 && parts.iter().all(|&part| !part.is_empty())
    }
}

impl Decode<'_, Postgres> for SlugString {
    fn decode(value: PgValueRef<'_>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let s = <String as Decode<Postgres>>::decode(value)?;
        SlugString::new(&s).map_err(|_| {
            Box::new(SlugError::InvalidFormat(s)) as Box<dyn std::error::Error + Send + Sync>
        })
    }
}

impl sqlx::Type<Postgres> for SlugString {
    fn type_info() -> PgTypeInfo {
        <String as sqlx::Type<Postgres>>::type_info()
    }
}

impl From<String> for SlugString {
    fn from(s: String) -> Self {
        SlugString(s)
    }
}

/// A slug to be retrieved and stored in the database, points to a resource like a paste or a file.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Slug {
    pub id: i32,
    pub file_id: Option<i32>,
    pub paste_id: Option<i32>,
    pub slug: SlugString,
    pub enabled: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

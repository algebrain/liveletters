use crate::DomainError;
use liveletters_utils::text::require_non_blank;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostBody(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentBody(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(u64);

fn require_non_blank_body(value: &str, field: &'static str) -> Result<String, DomainError> {
    require_non_blank(value, field).map_err(|_| DomainError::BlankBody(field))?;
    Ok(value.trim().to_owned())
}

impl PostBody {
    pub fn new(value: &str) -> Result<Self, DomainError> {
        Ok(Self(require_non_blank_body(value, "post_body")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl CommentBody {
    pub fn new(value: &str) -> Result<Self, DomainError> {
        Ok(Self(require_non_blank_body(value, "comment_body")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Timestamp {
    pub fn from_unix_seconds(value: u64) -> Self {
        Self(value)
    }

    pub fn as_unix_seconds(&self) -> u64 {
        self.0
    }
}

use crate::DomainError;
use liveletters_utils::text::require_non_blank;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PostId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommentId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccountId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventId(String);

fn require_identifier(value: &str, field: &'static str) -> Result<String, DomainError> {
    require_non_blank(value, field).map_err(|_| DomainError::BlankIdentifier(field))?;
    Ok(value.trim().to_owned())
}

impl PostId {
    pub fn new(value: &str) -> Result<Self, DomainError> {
        Ok(Self(require_identifier(value, "post_id")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl CommentId {
    pub fn new(value: &str) -> Result<Self, DomainError> {
        Ok(Self(require_identifier(value, "comment_id")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ResourceId {
    pub fn new(value: &str) -> Result<Self, DomainError> {
        Ok(Self(require_identifier(value, "resource_id")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AccountId {
    pub fn new(value: &str) -> Result<Self, DomainError> {
        Ok(Self(require_identifier(value, "account_id")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl EventId {
    pub fn new(value: &str) -> Result<Self, DomainError> {
        Ok(Self(require_identifier(value, "event_id")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

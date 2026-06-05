use crate::DomainError;

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

fn require_non_blank(value: &str, field: &'static str) -> Result<String, DomainError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(DomainError::BlankIdentifier(field));
    }

    Ok(trimmed.to_owned())
}

impl PostId {
    pub fn new(value: &str) -> Result<Self, DomainError> {
        Ok(Self(require_non_blank(value, "post_id")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl CommentId {
    pub fn new(value: &str) -> Result<Self, DomainError> {
        Ok(Self(require_non_blank(value, "comment_id")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ResourceId {
    pub fn new(value: &str) -> Result<Self, DomainError> {
        Ok(Self(require_non_blank(value, "resource_id")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AccountId {
    pub fn new(value: &str) -> Result<Self, DomainError> {
        Ok(Self(require_non_blank(value, "account_id")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl EventId {
    pub fn new(value: &str) -> Result<Self, DomainError> {
        Ok(Self(require_non_blank(value, "event_id")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

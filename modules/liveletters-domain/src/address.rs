use serde::{Deserialize, Serialize};

use crate::DomainError;
use liveletters_utils::{email::looks_like_email, text::require_non_blank};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceAddress(String);

impl ResourceAddress {
    pub fn new(value: &str) -> Result<Self, DomainError> {
        require_non_blank(value, "resource_address")
            .map_err(|_| DomainError::BlankIdentifier("resource_address"))?;
        let trimmed = value.trim();
        if !looks_like_email(trimmed) {
            return Err(DomainError::InvalidAddress);
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

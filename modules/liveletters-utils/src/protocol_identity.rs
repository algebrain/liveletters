use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use thiserror::Error;

use crate::{email::looks_like_email, text::require_non_blank};

/// Человек и его почтовый адрес в строковом формате протокола:
/// `Nickname <email@example.org>`.
///
/// В JSON протокольного сообщения этот тип сериализуется именно как строка,
/// чтобы `origin` и `source` оставались простыми и читаемыми.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolIdentity {
    nickname: String,
    email: String,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProtocolIdentityError {
    #[error("protocol identity has invalid wire format: {input}")]
    InvalidWireFormat { input: String },
    #[error("protocol identity nickname is blank")]
    BlankNickname,
    #[error("protocol identity email is blank")]
    BlankEmail,
    #[error("protocol identity email is invalid: {email}")]
    InvalidEmail { email: String },
}

impl ProtocolIdentity {
    /// Создает идентичность из уже разделенных имени и адреса.
    ///
    /// Значения обрезаются по краям, но внутри не переписываются.
    pub fn new(
        nickname: impl Into<String>,
        email: impl Into<String>,
    ) -> Result<Self, ProtocolIdentityError> {
        let nickname = normalize_part(nickname.into());
        let email = normalize_part(email.into());
        validate_nickname(&nickname)?;
        validate_email(&email)?;
        Ok(Self { nickname, email })
    }

    /// Разбирает строку вида `Nickname <email@example.org>`.
    pub fn parse(input: &str) -> Result<Self, ProtocolIdentityError> {
        let trimmed = input.trim();
        let Some(open) = trimmed.find('<') else {
            return Err(invalid_wire_format(input));
        };
        let Some(close) = trimmed.rfind('>') else {
            return Err(invalid_wire_format(input));
        };
        if close <= open {
            return Err(invalid_wire_format(input));
        }

        let nickname = &trimmed[..open];
        let email = &trimmed[open + 1..close];
        let tail = &trimmed[close + 1..];
        if !tail.trim().is_empty()
            || email.contains('<')
            || email.contains('>')
            || nickname.contains('<')
            || nickname.contains('>')
        {
            return Err(invalid_wire_format(input));
        }

        Self::new(nickname, email)
    }

    pub fn nickname(&self) -> &str {
        &self.nickname
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    /// Возвращает строку для JSON-поля `origin` или `source`.
    pub fn to_wire_string(&self) -> String {
        format!("{} <{}>", self.nickname, self.email)
    }
}

impl fmt::Display for ProtocolIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_wire_string())
    }
}

impl Serialize for ProtocolIdentity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_wire_string())
    }
}

impl<'de> Deserialize<'de> for ProtocolIdentity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(de::Error::custom)
    }
}

fn normalize_part(value: String) -> String {
    value.trim().to_owned()
}

fn validate_nickname(value: &str) -> Result<(), ProtocolIdentityError> {
    require_non_blank(value, "nickname").map_err(|_| ProtocolIdentityError::BlankNickname)?;
    Ok(())
}

fn validate_email(value: &str) -> Result<(), ProtocolIdentityError> {
    require_non_blank(value, "email").map_err(|_| ProtocolIdentityError::BlankEmail)?;
    if value.contains('<') || value.contains('>') || !looks_like_email(value) {
        return Err(ProtocolIdentityError::InvalidEmail {
            email: value.to_owned(),
        });
    }
    Ok(())
}

fn invalid_wire_format(input: &str) -> ProtocolIdentityError {
    ProtocolIdentityError::InvalidWireFormat {
        input: input.to_owned(),
    }
}

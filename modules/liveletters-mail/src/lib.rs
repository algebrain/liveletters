//! `liveletters-mail` объединяет конфигурацию почты, разбор/сборку
//! MIME-сообщений (через `liveletters-mime`) и опциональные сетевые
//! транспорты IMAP/SMTP. Сетевые части подключаются под признаком
//! `network` и зависят от `native-tls` только в этом режиме.
mod config;
mod errors;
mod retry;
mod status;
#[cfg(feature = "network")]
pub mod transport;

pub use config::{ImapMailboxConfig, MailAuth, MailSecurity, SmtpTransportConfig};
pub use errors::TransportError;
pub use liveletters_mime::{
    ExtractedMailParts, MimeError, OutgoingEmail, ParsedEmail, ReceivedEmail, build_protocol_email,
    decode_protocol_message, extract_liveletters_parts, parse_email,
};
pub use retry::MailRetryPolicy;
pub use status::{FetchBatch, FetchStatus, MailboxCursor, SendStatus};

#[cfg(feature = "network")]
pub use transport::{ConfiguredImapMailbox, ConfiguredSmtpTransport};

pub fn crate_name() -> &'static str {
    "liveletters-mail"
}

impl From<MimeError> for TransportError {
    fn from(error: MimeError) -> Self {
        match error {
            MimeError::Protocol(message) => Self::Protocol(message),
            MimeError::InvalidEmailFormat(message) => Self::InvalidEmailFormat(message),
            MimeError::MissingHumanReadablePart => Self::MissingHumanReadablePart,
            MimeError::MissingTechnicalPart => Self::MissingTechnicalPart,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::crate_name;

    #[test]
    fn exposes_crate_name() {
        assert_eq!(crate_name(), "liveletters-mail");
    }
}

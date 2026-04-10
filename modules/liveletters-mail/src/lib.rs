mod config;
mod errors;
mod imap;
mod message;
mod mime;
mod parser;
mod retry;
mod smtp;
mod status;

use liveletters_protocol::{ProtocolMessage, ProtocolError, decode_message, encode_message};

pub use config::{ImapMailboxConfig, MailAuth, MailSecurity, SmtpTransportConfig};
pub use errors::TransportError;
pub use imap::{ConfiguredImapMailbox, InMemoryImapMailbox};
pub use message::{ExtractedMailParts, OutgoingEmail, ParsedEmail, ReceivedEmail};
pub use mime::extract_liveletters_parts;
pub use parser::parse_email;
pub use retry::MailRetryPolicy;
pub use smtp::{ConfiguredSmtpTransport, InMemorySmtpTransport};
pub use status::{FetchBatch, FetchStatus, MailboxCursor, SendStatus};

pub fn crate_name() -> &'static str {
    "liveletters-mail"
}

pub fn build_protocol_email(
    from: &str,
    to: &str,
    subject: &str,
    protocol_message: &ProtocolMessage,
) -> Result<OutgoingEmail, TransportError> {
    let boundary = "liveletters-boundary";
    let technical_payload =
        encode_message(protocol_message).map_err(|error| TransportError::Protocol(format_protocol_error(error)))?;

    let raw_message = format!(
        "From: {from}\nTo: {to}\nSubject: {subject}\nMIME-Version: 1.0\nContent-Type: multipart/mixed; boundary=\"{boundary}\"\n\n--{boundary}\nContent-Type: text/plain; charset=\"utf-8\"\n\n{}\n--{boundary}\nContent-Type: application/json\n\n{}\n--{boundary}--\n",
        protocol_message.human_readable_body(),
        technical_payload
    );

    Ok(OutgoingEmail {
        from: from.to_owned(),
        to: to.to_owned(),
        subject: subject.to_owned(),
        raw_message,
    })
}

pub fn decode_protocol_message(input: &str) -> Result<ProtocolMessage, TransportError> {
    decode_message(input).map_err(|error| TransportError::Protocol(format_protocol_error(error)))
}

fn format_protocol_error(error: ProtocolError) -> String {
    match error {
        ProtocolError::BlankEnvelopeField(field) => format!("blank envelope field: {field}"),
        ProtocolError::BlankHumanReadableBody => "blank human readable body".to_owned(),
        ProtocolError::MalformedJson(message) => format!("malformed json: {message}"),
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

mod build;
mod error;
mod limits;
mod message;
mod mime;
mod parser;

pub use build::{build_protocol_email, decode_protocol_message};
pub use error::MimeError;
pub use limits::MimeLimits;
pub use message::{ExtractedMailParts, OutgoingEmail, ParsedEmail, ReceivedEmail};
pub use mime::{extract_liveletters_parts, extract_liveletters_parts_with_limits};
pub use parser::{parse_email, parse_email_with_limits};

pub fn crate_name() -> &'static str {
    "liveletters-mime"
}

#[cfg(test)]
mod tests {
    use super::crate_name;

    #[test]
    fn exposes_crate_name() {
        assert_eq!(crate_name(), "liveletters-mime");
    }
}

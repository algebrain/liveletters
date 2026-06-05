mod build;
mod error;
mod message;
mod mime;
mod parser;

pub use build::{build_protocol_email, decode_protocol_message};
pub use error::MimeError;
pub use message::{ExtractedMailParts, OutgoingEmail, ParsedEmail, ReceivedEmail};
pub use mime::extract_liveletters_parts;
pub use parser::parse_email;

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

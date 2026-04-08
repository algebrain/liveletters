mod codec;
mod envelope;
mod error;
mod message;
mod payload;

pub use codec::{decode_message, encode_message};
pub use envelope::MessageEnvelope;
pub use error::ProtocolError;
pub use message::ProtocolMessage;
pub use payload::DomainEventPayload;

pub fn crate_name() -> &'static str {
    "liveletters-protocol"
}

#[cfg(test)]
mod tests {
    use super::crate_name;

    #[test]
    fn exposes_crate_name() {
        assert_eq!(crate_name(), "liveletters-protocol");
    }
}

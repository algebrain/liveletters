use crate::{ProtocolError, ProtocolMessage};

pub fn encode_message(message: &ProtocolMessage) -> Result<String, ProtocolError> {
    serde_json::to_string_pretty(message).map_err(|error| ProtocolError::MalformedJson(error.to_string()))
}

pub fn decode_message(input: &str) -> Result<ProtocolMessage, ProtocolError> {
    serde_json::from_str(input).map_err(|error| ProtocolError::MalformedJson(error.to_string()))
}

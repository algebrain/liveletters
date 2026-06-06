use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use liveletters_protocol::{ProtocolError, ProtocolMessage, decode_message, encode_message};

use crate::{MimeError, OutgoingEmail};

pub fn build_protocol_email(
    from: &str,
    to: &str,
    subject: &str,
    protocol_message: &ProtocolMessage,
) -> Result<OutgoingEmail, MimeError> {
    let technical_payload = encode_message(protocol_message)
        .map_err(|error| MimeError::Protocol(format_protocol_error(error)))?;
    let encoded_payload = URL_SAFE_NO_PAD.encode(technical_payload.as_bytes());

    let raw_message = format!(
        "From: {from}\nTo: {to}\nSubject: {subject}\nX-LiveLetters-Protocol: v1\nContent-Type: text/plain; charset=\"utf-8\"\n\n{}\n\n-- \nLiveLetters-Protocol: v1\nLiveLetters-Payload: {encoded_payload}\n",
        protocol_message.human_readable_body(),
    );

    Ok(OutgoingEmail {
        from: from.to_owned(),
        to: to.to_owned(),
        subject: subject.to_owned(),
        raw_message,
    })
}

pub fn decode_protocol_message(input: &str) -> Result<ProtocolMessage, MimeError> {
    decode_message(input).map_err(|error| MimeError::Protocol(format_protocol_error(error)))
}

fn format_protocol_error(error: ProtocolError) -> String {
    match error {
        ProtocolError::BlankEnvelopeField(field) => format!("blank envelope field: {field}"),
        ProtocolError::BlankHumanReadableBody => "blank human readable body".to_owned(),
        ProtocolError::MalformedJson(message) => format!("malformed json: {message}"),
    }
}

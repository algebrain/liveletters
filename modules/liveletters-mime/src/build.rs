use liveletters_protocol::{ProtocolError, ProtocolMessage, decode_message, encode_message};

use crate::{MimeError, OutgoingEmail};

pub fn build_protocol_email(
    from: &str,
    to: &str,
    subject: &str,
    protocol_message: &ProtocolMessage,
) -> Result<OutgoingEmail, MimeError> {
    let boundary = "liveletters-boundary";
    let technical_payload = encode_message(protocol_message)
        .map_err(|error| MimeError::Protocol(format_protocol_error(error)))?;

    let raw_message = format!(
        "From: {from}\nTo: {to}\nSubject: {subject}\nX-LiveLetters-Protocol: v1\nMIME-Version: 1.0\nContent-Type: multipart/mixed; boundary=\"{boundary}\"\n\n--{boundary}\nContent-Type: text/plain; charset=\"utf-8\"\n\n{}\n--{boundary}\nContent-Type: application/json\n\n{}\n--{boundary}--\n",
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

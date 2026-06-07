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

    let raw_message = format!(
        "From: {from}\n\
         To: {to}\n\
         Subject: {subject}\n\
         X-LiveLetters-Protocol: v1\n\
         MIME-Version: 1.0\n\
         Content-Type: multipart/mixed; boundary=\"{BOUNDARY}\"\n\
         \n\
         --{BOUNDARY}\n\
         Content-Type: text/plain; charset=\"utf-8\"\n\
         \n\
         {human}\n\
         --{BOUNDARY}\n\
         Content-Type: application/json; name=\"{JSON_FILENAME}\"\n\
         Content-Disposition: attachment; filename=\"{JSON_FILENAME}\"\n\
         \n\
         {technical_payload}\n\
         --{BOUNDARY}--\n",
        human = protocol_message.human_readable_body(),
    );

    Ok(OutgoingEmail {
        from: from.to_owned(),
        to: to.to_owned(),
        subject: subject.to_owned(),
        raw_message,
    })
}

const BOUNDARY: &str = "liveletters-boundary";
const JSON_FILENAME: &str = "liveletters.json";

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

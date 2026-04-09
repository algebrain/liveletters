use crate::{ParsedEmail, TransportError};

pub fn parse_email(raw_email: &str) -> Result<ParsedEmail, TransportError> {
    let normalized = raw_email.replace("\r\n", "\n");
    let Some((header_block, body)) = normalized.split_once("\n\n") else {
        return Err(TransportError::InvalidEmailFormat(
            "email must contain headers and body",
        ));
    };

    let mut headers = Vec::new();
    for line in header_block.lines() {
        let Some((name, value)) = line.split_once(':') else {
            return Err(TransportError::InvalidEmailFormat(
                "header line must contain colon",
            ));
        };

        headers.push((name.trim().to_owned(), value.trim().to_owned()));
    }

    Ok(ParsedEmail::new(headers, body.to_owned()))
}

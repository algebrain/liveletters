use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

use crate::{ExtractedMailParts, MimeError, ParsedEmail};

const INLINE_PROTOCOL_MARKER: &str = "\n-- \nLiveLetters-Protocol: v1\nLiveLetters-Payload: ";

pub fn extract_liveletters_parts(parsed: &ParsedEmail) -> Result<ExtractedMailParts, MimeError> {
    let Some(content_type) = parsed.header("Content-Type") else {
        return Err(MimeError::InvalidEmailFormat("missing Content-Type header"));
    };

    if content_type.contains("text/plain") {
        return extract_inline_protocol(parsed.body());
    }

    if !content_type.contains("multipart/") {
        return Err(MimeError::InvalidEmailFormat(
            "expected multipart Content-Type",
        ));
    }

    let boundary = extract_boundary(content_type)?;
    let boundary_marker = format!("--{boundary}");

    let mut human_readable_body = None;
    let mut technical_body = None;

    for chunk in parsed.body().split(&boundary_marker).skip(1) {
        let part = chunk.trim();
        if part.is_empty() || part == "--" {
            continue;
        }

        let part = part.strip_suffix("--").unwrap_or(part).trim();
        let Some((header_block, body)) = part.split_once("\n\n") else {
            continue;
        };

        let body = body.trim();
        let part_content_type = header_block
            .lines()
            .find_map(|line| line.split_once(':'))
            .filter(|(name, _)| name.trim().eq_ignore_ascii_case("Content-Type"))
            .map(|(_, value)| value.trim().to_owned());

        match part_content_type.as_deref() {
            Some(value) if value.contains("text/plain") => {
                human_readable_body = Some(body.to_owned());
            }
            Some(value) if value.contains("application/json") => {
                technical_body = Some(body.to_owned());
            }
            _ => {}
        }
    }

    Ok(ExtractedMailParts::new(
        human_readable_body.ok_or(MimeError::MissingHumanReadablePart)?,
        technical_body.ok_or(MimeError::MissingTechnicalPart)?,
    ))
}

fn extract_inline_protocol(raw_body: &str) -> Result<ExtractedMailParts, MimeError> {
    let Some((human, encoded_payload)) = raw_body.rsplit_once(INLINE_PROTOCOL_MARKER) else {
        return Err(MimeError::MissingTechnicalPart);
    };

    let payload = URL_SAFE_NO_PAD
        .decode(encoded_payload.trim())
        .map_err(|_| MimeError::InvalidEmailFormat("invalid LiveLetters payload encoding"))?;
    let technical_body = String::from_utf8(payload)
        .map_err(|_| MimeError::InvalidEmailFormat("LiveLetters payload is not UTF-8"))?;

    Ok(ExtractedMailParts::new(
        human.trim_end().to_owned(),
        technical_body,
    ))
}

fn extract_boundary(content_type: &str) -> Result<String, MimeError> {
    let Some((_, tail)) = content_type.split_once("boundary=") else {
        return Err(MimeError::InvalidEmailFormat(
            "multipart Content-Type must include boundary",
        ));
    };

    Ok(tail.trim().trim_matches('"').to_owned())
}

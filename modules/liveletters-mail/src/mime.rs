use crate::{ExtractedMailParts, ParsedEmail, TransportError};

pub fn extract_liveletters_parts(parsed: &ParsedEmail) -> Result<ExtractedMailParts, TransportError> {
    let Some(content_type) = parsed.header("Content-Type") else {
        return Err(TransportError::InvalidEmailFormat("missing Content-Type header"));
    };

    if !content_type.contains("multipart/") {
        return Err(TransportError::InvalidEmailFormat(
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
        human_readable_body.ok_or(TransportError::MissingHumanReadablePart)?,
        technical_body.ok_or(TransportError::MissingTechnicalPart)?,
    ))
}

fn extract_boundary(content_type: &str) -> Result<String, TransportError> {
    let Some((_, tail)) = content_type.split_once("boundary=") else {
        return Err(TransportError::InvalidEmailFormat(
            "multipart Content-Type must include boundary",
        ));
    };

    Ok(tail.trim().trim_matches('"').to_owned())
}

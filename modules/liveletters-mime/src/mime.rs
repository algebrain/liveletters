use crate::{ExtractedMailParts, MimeError, ParsedEmail};

pub fn extract_liveletters_parts(parsed: &ParsedEmail) -> Result<ExtractedMailParts, MimeError> {
    let mail = mailparse::parse_mail(parsed.raw().as_bytes())
        .map_err(|_| MimeError::InvalidEmailFormat("cannot parse email as MIME"))?;

    if !mail.ctype.mimetype.starts_with("multipart/") {
        return Err(MimeError::InvalidEmailFormat("expected multipart message"));
    }

    let mut human_readable_body = None;
    let mut technical_body = None;

    for part in &mail.subparts {
        if part.ctype.mimetype.contains("text/plain") {
            let body = part
                .get_body()
                .map_err(|_| MimeError::InvalidEmailFormat("cannot decode text body"))?;
            human_readable_body = Some(body);
        } else if part.ctype.mimetype.contains("application/json") {
            let raw = part
                .get_body_raw()
                .map_err(|_| MimeError::InvalidEmailFormat("cannot decode json body"))?;
            let body = String::from_utf8(raw.to_vec())
                .map_err(|_| MimeError::InvalidEmailFormat("json body is not valid utf-8"))?;
            technical_body = Some(body);
        }
    }

    Ok(ExtractedMailParts::new(
        human_readable_body.ok_or(MimeError::MissingHumanReadablePart)?,
        technical_body.ok_or(MimeError::MissingTechnicalPart)?,
    ))
}

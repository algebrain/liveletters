use mailparse::{MailHeaderMap, ParsedMail};

use crate::{ExtractedMailParts, MimeError, MimeLimits, ParsedEmail};

pub fn extract_liveletters_parts(parsed: &ParsedEmail) -> Result<ExtractedMailParts, MimeError> {
    extract_liveletters_parts_with_limits(parsed, MimeLimits::default())
}

pub fn extract_liveletters_parts_with_limits(
    parsed: &ParsedEmail,
    limits: MimeLimits,
) -> Result<ExtractedMailParts, MimeError> {
    let mail = mailparse::parse_mail(parsed.raw().as_bytes())
        .map_err(|_| MimeError::InvalidEmailFormat("cannot parse email as MIME"))?;

    if !mail.ctype.mimetype.starts_with("multipart/") {
        return Err(MimeError::InvalidEmailFormat("expected multipart message"));
    }

    validate_part_count_and_depth(&mail, limits)?;

    let mut human_readable_body: Option<String> = None;
    let mut technical_body: Option<String> = None;

    for part in &mail.subparts {
        if is_text_plain(part) {
            if human_readable_body.is_some() {
                return Err(MimeError::InvalidEmailFormat(
                    "duplicate human readable part",
                ));
            }
            let body = part
                .get_body()
                .map_err(|_| MimeError::InvalidEmailFormat("cannot decode text body"))?;
            if body.len() > limits.max_human_bytes {
                return Err(MimeError::InvalidEmailFormat(
                    "human readable part exceeds size limit",
                ));
            }
            human_readable_body = Some(body);
        } else if is_application_json(part) {
            if !is_liveletters_json(part) {
                return Err(MimeError::InvalidEmailFormat(
                    "liveletters json part must be named liveletters.json",
                ));
            }
            if technical_body.is_some() {
                return Err(MimeError::InvalidEmailFormat(
                    "duplicate liveletters json part",
                ));
            }
            let raw = part
                .get_body_raw()
                .map_err(|_| MimeError::InvalidEmailFormat("cannot decode json body"))?;
            if raw.len() > limits.max_json_bytes {
                return Err(MimeError::InvalidEmailFormat(
                    "liveletters json exceeds size limit",
                ));
            }
            let body = String::from_utf8(raw.to_vec())
                .map_err(|_| MimeError::InvalidEmailFormat("json body is not valid utf-8"))?;
            technical_body = Some(body);
        } else {
            return Err(MimeError::InvalidEmailFormat(
                "attachments require manifest",
            ));
        }
    }

    Ok(ExtractedMailParts::new(
        human_readable_body.ok_or(MimeError::MissingHumanReadablePart)?,
        technical_body.ok_or(MimeError::MissingTechnicalPart)?,
    ))
}

fn validate_part_count_and_depth(
    mail: &ParsedMail<'_>,
    limits: MimeLimits,
) -> Result<(), MimeError> {
    let mut count = 0;
    count_parts(mail, 0, limits, &mut count)
}

fn count_parts(
    mail: &ParsedMail<'_>,
    depth: usize,
    limits: MimeLimits,
    count: &mut usize,
) -> Result<(), MimeError> {
    if depth > limits.max_depth {
        return Err(MimeError::InvalidEmailFormat("mime depth exceeds limit"));
    }
    *count += 1;
    if *count > limits.max_parts {
        return Err(MimeError::InvalidEmailFormat("too many mime parts"));
    }
    for part in &mail.subparts {
        count_parts(part, depth + 1, limits, count)?;
    }
    Ok(())
}

fn is_text_plain(part: &ParsedMail<'_>) -> bool {
    part.ctype.mimetype.eq_ignore_ascii_case("text/plain")
}

fn is_application_json(part: &ParsedMail<'_>) -> bool {
    part.ctype.mimetype.eq_ignore_ascii_case("application/json")
}

fn is_liveletters_json(part: &ParsedMail<'_>) -> bool {
    let content_type_name = part
        .ctype
        .params
        .get("name")
        .is_some_and(|value| value == "liveletters.json");
    let disposition_filename = part
        .get_headers()
        .get_first_value("Content-Disposition")
        .and_then(|value| filename_from_content_disposition(&value))
        .is_some_and(|value| value == "liveletters.json");

    content_type_name || disposition_filename
}

fn filename_from_content_disposition(value: &str) -> Option<String> {
    value.split(';').find_map(|part| {
        let (key, value) = part.trim().split_once('=')?;
        if !key.trim().eq_ignore_ascii_case("filename") {
            return None;
        }
        Some(value.trim().trim_matches('"').to_owned())
    })
}

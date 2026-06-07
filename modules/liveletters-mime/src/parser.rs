use crate::{MimeError, ParsedEmail};

pub fn parse_email(raw_email: &str) -> Result<ParsedEmail, MimeError> {
    let normalized = raw_email.replace("\r\n", "\n");
    if !normalized.contains("\n\n") {
        return Err(MimeError::InvalidEmailFormat(
            "email must contain headers and body",
        ));
    }

    let mail = mailparse::parse_mail(normalized.as_bytes())
        .map_err(|_| MimeError::InvalidEmailFormat("cannot parse email"))?;

    let mut headers = Vec::new();
    for h in &mail.headers {
        headers.push((h.get_key().to_owned(), h.get_value()));
    }

    Ok(ParsedEmail::new(normalized, headers))
}

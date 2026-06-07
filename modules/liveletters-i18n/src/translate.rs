use crate::{I18nError, Locale, templates};

#[derive(Debug, Clone, Copy)]
pub struct Vars<'a>(pub &'a [(&'a str, &'a str)]);

pub fn translate(key: &str, locale: Locale, vars: Vars<'_>) -> Result<String, I18nError> {
    let template =
        templates::template(key, locale).ok_or_else(|| I18nError::UnknownKey(key.to_owned()))?;

    let mut result = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '%' {
            result.push(c);
            continue;
        }
        let mut name = String::new();
        let mut found_close = false;
        while let Some(&next) = chars.peek() {
            if next == '%' {
                chars.next();
                found_close = true;
                break;
            }
            name.push(next);
            chars.next();
        }
        if !found_close {
            return Err(I18nError::MissingVariable {
                key: key.to_owned(),
                name: format!("unterminated %{name}"),
            });
        }
        if name.is_empty() {
            return Err(I18nError::MissingVariable {
                key: key.to_owned(),
                name: "<empty>".to_owned(),
            });
        }
        let value = vars
            .0
            .iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| *v)
            .ok_or_else(|| I18nError::MissingVariable {
                key: key.to_owned(),
                name: name.clone(),
            })?;
        result.push_str(value);
    }
    Ok(result)
}

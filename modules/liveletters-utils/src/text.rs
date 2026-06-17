use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("field `{field}` is blank")]
pub struct NonBlankError {
    pub field: &'static str,
}

/// Проверяет, что строка не пустая и не состоит только из пробелов.
///
/// Возвращает исходную строку без обрезания: вызывающий сам решает, нужно ли
/// сохранять пробелы как значимые данные.
pub fn require_non_blank<'a>(
    value: &'a str,
    field: &'static str,
) -> Result<&'a str, NonBlankError> {
    if value.trim().is_empty() {
        return Err(NonBlankError { field });
    }
    Ok(value)
}

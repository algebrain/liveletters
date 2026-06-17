/// Простая проверка формы почтового адреса для уже существующих сценариев.
///
/// Это не RFC-парсер. Контракт намеренно узкий: одна `@`, непустая локальная
/// часть, непустой домен, без пробельных символов внутри.
pub fn looks_like_email(value: &str) -> bool {
    email_local_part(value).is_some()
}

/// Возвращает локальную часть адреса до `@`, если адрес имеет простую форму.
pub fn email_local_part(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.chars().any(char::is_whitespace) {
        return None;
    }
    let (local, domain) = trimmed.split_once('@')?;
    if local.is_empty() || domain.is_empty() || domain.contains('@') {
        return None;
    }
    Some(local)
}

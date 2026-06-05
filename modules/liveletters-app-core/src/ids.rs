//! Генерация идентификаторов записей и комментариев на основе системного времени.
//!
//! Формат: префикс + миллисекунды UNIX. Коллизии при ручном вводе исключены
//! (CLI передаёт готовый ID при тестировании), а для пользовательского
//! темпа создания записей (одна в несколько секунд) миллисекундного
//! разрешения достаточно.

use std::time::{SystemTime, UNIX_EPOCH};

use liveletters_domain::DomainError;

pub fn unix_millis_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn new_post_id() -> Result<String, DomainError> {
    Ok(format!("post-{}", unix_millis_now()))
}

pub fn new_comment_id() -> Result<String, DomainError> {
    Ok(format!("comment-{}", unix_millis_now()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_millis_now_is_nonzero() {
        assert!(unix_millis_now() > 1_700_000_000_000);
    }

    #[test]
    fn new_post_id_starts_with_post_prefix() {
        let id = new_post_id().unwrap();
        assert!(id.starts_with("post-"));
        let suffix = &id[5..];
        assert!(suffix.parse::<u64>().is_ok());
    }

    #[test]
    fn new_comment_id_starts_with_comment_prefix() {
        let id = new_comment_id().unwrap();
        assert!(id.starts_with("comment-"));
        let suffix = &id[8..];
        assert!(suffix.parse::<u64>().is_ok());
    }

    #[test]
    fn ids_increase_over_time() {
        let first = new_post_id().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let second = new_post_id().unwrap();
        assert!(first < second);
    }
}

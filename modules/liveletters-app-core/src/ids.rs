//! Генерация идентификаторов записей и комментариев на основе UUID v7.
//!
//! Формат: префикс (`post-` / `comment-`) + UUID v7 без дефисов
//! (32 hex-символа). UUID v7 — time-ordered, 62 случайных бита,
//! глобально уникален между разными машинами и пользователями.

use std::time::{SystemTime, UNIX_EPOCH};

use liveletters_domain::DomainError;
use uuid::Uuid;

pub fn unix_millis_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn new_post_id() -> Result<String, DomainError> {
    Ok(format!("post-{}", Uuid::now_v7().simple()))
}

pub fn new_comment_id() -> Result<String, DomainError> {
    Ok(format!("comment-{}", Uuid::now_v7().simple()))
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
    }

    #[test]
    fn new_comment_id_starts_with_comment_prefix() {
        let id = new_comment_id().unwrap();
        assert!(id.starts_with("comment-"));
    }

    #[test]
    fn two_post_ids_in_same_millisecond_are_different() {
        // 100 вызовов подряд в один тик — все различны за счёт случайных бит.
        let ids: Vec<String> = (0..100).map(|_| new_post_id().unwrap()).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len(), "должны быть все уникальны");
    }

    #[test]
    fn two_comment_ids_in_same_millisecond_are_different() {
        let ids: Vec<String> = (0..100).map(|_| new_comment_id().unwrap()).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len(), "должны быть все уникальны");
    }

    #[test]
    fn post_id_suffix_is_not_just_timestamp() {
        // Суффикс — 32 hex-символа (UUID v7 без дефисов), не одно число.
        let id = new_post_id().unwrap();
        let suffix = &id[5..];
        assert_eq!(suffix.len(), 32, "UUID v7 без дефисов = 32 hex");
        assert!(
            suffix.parse::<u64>().is_err(),
            "суффикс не должен быть одним числом-таймстампом: {suffix}"
        );
    }

    #[test]
    fn comment_id_suffix_is_not_just_timestamp() {
        let id = new_comment_id().unwrap();
        let suffix = &id[8..];
        assert_eq!(suffix.len(), 32);
        assert!(suffix.parse::<u64>().is_err());
    }
}

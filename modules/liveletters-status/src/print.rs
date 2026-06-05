//! Печать сводки состояния домашнего каталога.

use liveletters_output::{format_unix_iso8601_utc, print_kv};

/// Подсчёты, которые показывает команда `lltt status`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusCounts {
    pub posts: u64,
    pub comments: u64,
    pub deferred: u64,
    pub outbox: u64,
    pub last_activity: Option<u64>,
}

pub fn print_status(counts: &StatusCounts) {
    let last = counts
        .last_activity
        .map(format_unix_iso8601_utc)
        .unwrap_or_else(|| "нет активности".to_owned());
    print_kv(&[
        ("постов", &counts.posts.to_string()),
        ("комментариев", &counts.comments.to_string()),
        ("отложенных", &counts.deferred.to_string()),
        ("исходящих", &counts.outbox.to_string()),
        ("последняя активность", &last),
    ]);
}

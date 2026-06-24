//! Лимиты и политика удержания для приёма событий.
//!
//! Определения типов тривиальны и живут здесь, рядом с движком, чтобы
//! `liveletters-config` мог их сериализовать через прямую зависимость
//! `config -> sync` (обратной зависимости нет, цикл не возникает).

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

/// Квоты на один `ingest_batch` (один `sync pull`).
///
/// Превышение любой квоты приводит к исходу
/// [`crate::SyncMessageOutcome::RateLimited`] вместо применения события.
/// Нормальный ручной поток (несколько постов/комментариев/подписок) не должен
/// задевать эти значения; они защищают только от аномального всплеска.
///
/// Каждое поле несёт собственный serde-default: частичный файл
/// `users/<name>/config.toml` дополняется кодовыми значениями для
/// отсутствующих ключей.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestLimits {
    #[serde(default = "default_max_deferred_total")]
    pub max_deferred_total: usize,
    #[serde(default = "default_max_deferred_per_origin")]
    pub max_deferred_per_origin: usize,
    #[serde(default = "default_max_new_authors_per_batch")]
    pub max_new_authors_per_batch: usize,
    #[serde(default = "default_max_auto_responses_per_batch")]
    pub max_auto_responses_per_batch: usize,
    #[serde(default = "default_max_events_per_origin")]
    pub max_events_per_origin: usize,
    #[serde(default = "default_max_events_per_source")]
    pub max_events_per_source: usize,
}

impl Default for IngestLimits {
    fn default() -> Self {
        Self {
            max_deferred_total: default_max_deferred_total(),
            max_deferred_per_origin: default_max_deferred_per_origin(),
            max_new_authors_per_batch: default_max_new_authors_per_batch(),
            max_auto_responses_per_batch: default_max_auto_responses_per_batch(),
            max_events_per_origin: default_max_events_per_origin(),
            max_events_per_source: default_max_events_per_source(),
        }
    }
}

fn default_max_deferred_total() -> usize {
    100
}
fn default_max_deferred_per_origin() -> usize {
    10
}
fn default_max_new_authors_per_batch() -> usize {
    50
}
fn default_max_auto_responses_per_batch() -> usize {
    20
}
fn default_max_events_per_origin() -> usize {
    50
}
fn default_max_events_per_source() -> usize {
    50
}

impl IngestLimits {
    /// Все квоты равны `usize::MAX`: применяется при локальной переобработке
    /// `reprocess_deferred`, где нет входящего трафика и подсчёт бессмысленен.
    pub fn disabled() -> Self {
        Self {
            max_deferred_total: usize::MAX,
            max_deferred_per_origin: usize::MAX,
            max_new_authors_per_batch: usize::MAX,
            max_auto_responses_per_batch: usize::MAX,
            max_events_per_origin: usize::MAX,
            max_events_per_source: usize::MAX,
        }
    }
}

/// Политика удержания `raw_messages`. Действует между pull-ами (на стороне БД),
/// в отличие от [`IngestLimits`], которые считаются на один батч.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionPolicy {
    #[serde(default = "default_raw_messages_ttl_days")]
    pub raw_messages_ttl_days: u32,
    #[serde(default = "default_raw_messages_max_kept")]
    pub raw_messages_max_kept: usize,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            raw_messages_ttl_days: default_raw_messages_ttl_days(),
            raw_messages_max_kept: default_raw_messages_max_kept(),
        }
    }
}

fn default_raw_messages_ttl_days() -> u32 {
    14
}
fn default_raw_messages_max_kept() -> usize {
    5000
}

/// Счётчики одного `ingest_batch`. Не персистятся: живут только на время
/// одного вызова `ingest_batch` / `reprocess_deferred`.
#[derive(Default)]
pub(crate) struct BatchCounters {
    pub deferred_added: usize,
    pub new_authors: usize,
    pub auto_responses: usize,
    pub events_per_origin: HashMap<String, usize>,
    pub events_per_source: HashMap<String, usize>,
    pub deferred_per_origin: HashMap<String, usize>,
    pub known_authors: HashSet<String>,
}

impl BatchCounters {
    pub(crate) fn from_known(authors: impl IntoIterator<Item = String>) -> Self {
        let mut c = Self::default();
        c.known_authors.extend(authors);
        c
    }

    /// Регистрирует принятие письма в работу (не malformed/duplicate) и
    /// возвращает `Deny`, если превышен per-origin/per-source лимит событий.
    pub(crate) fn observe_event(
        &mut self,
        limits: IngestLimits,
        origin_email: &str,
        source_email: &str,
    ) -> QuotaCheck {
        let n_origin = bump(&mut self.events_per_origin, origin_email.to_owned());
        if n_origin > limits.max_events_per_origin {
            return QuotaCheck::Deny("events_per_origin_quota_exceeded");
        }
        if source_email != origin_email {
            let n_source = bump(&mut self.events_per_source, source_email.to_owned());
            if n_source > limits.max_events_per_source {
                return QuotaCheck::Deny("events_per_source_quota_exceeded");
            }
        } else {
            // source == origin: уже учли в origin-счётчике.
            self.events_per_source
                .insert(source_email.to_owned(), n_origin);
        }
        QuotaCheck::Allow
    }

    /// Решает, можно ли добавить нового автора. Возвращает `Deny`, если автор
    /// новый и превышен `max_new_authors_per_batch`. При `Allow` новый автор
    /// фиксируется в `known_authors`.
    pub(crate) fn check_new_author(&mut self, limits: IngestLimits, email: &str) -> QuotaCheck {
        if self.known_authors.contains(email) {
            return QuotaCheck::Allow;
        }
        if self.new_authors >= limits.max_new_authors_per_batch {
            return QuotaCheck::Deny("new_authors_quota_exceeded");
        }
        self.new_authors += 1;
        self.known_authors.insert(email.to_owned());
        QuotaCheck::Allow
    }

    /// Решает, можно ли отложить ещё одно событие.
    pub(crate) fn check_deferred(
        &mut self,
        limits: IngestLimits,
        total_in_db: u64,
        origin_email: &str,
    ) -> QuotaCheck {
        if total_in_db >= limits.max_deferred_total as u64 {
            return QuotaCheck::Deny("deferred_total_quota_exceeded");
        }
        let per_origin = bump(&mut self.deferred_per_origin, origin_email.to_owned());
        if per_origin > limits.max_deferred_per_origin {
            return QuotaCheck::Deny("deferred_per_origin_quota_exceeded");
        }
        QuotaCheck::Allow
    }

    /// Решает, можно ли сформировать ещё один автоматический ответ.
    pub(crate) fn check_auto_response(&mut self, limits: IngestLimits) -> QuotaCheck {
        if self.auto_responses >= limits.max_auto_responses_per_batch {
            return QuotaCheck::Deny("auto_response_quota_exceeded");
        }
        self.auto_responses += 1;
        QuotaCheck::Allow
    }

    /// Отмечает, что событие было отложено (для согласованности счётчиков,
    /// если внешняя проверка разрешила deferred).
    pub(crate) fn note_deferred_added(&mut self) {
        self.deferred_added += 1;
    }
}

fn bump(map: &mut HashMap<String, usize>, key: String) -> usize {
    let n = map.entry(key).or_insert(0);
    *n += 1;
    *n
}

#[derive(Debug)]
pub(crate) enum QuotaCheck {
    Allow,
    Deny(&'static str),
}

impl QuotaCheck {
    pub(crate) fn denied_reason(&self) -> Option<&'static str> {
        match self {
            QuotaCheck::Allow => None,
            QuotaCheck::Deny(reason) => Some(reason),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> IngestLimits {
        IngestLimits {
            max_deferred_total: 3,
            max_deferred_per_origin: 2,
            max_new_authors_per_batch: 2,
            max_auto_responses_per_batch: 1,
            max_events_per_origin: 2,
            max_events_per_source: 2,
        }
    }

    #[test]
    fn observe_event_caps_per_origin() {
        let mut c = BatchCounters::default();
        let l = limits();
        assert!(matches!(
            c.observe_event(l, "a@x", "a@x"),
            QuotaCheck::Allow
        ));
        assert!(matches!(
            c.observe_event(l, "a@x", "a@x"),
            QuotaCheck::Allow
        ));
        assert!(c.observe_event(l, "a@x", "a@x").denied_reason().is_some());
    }

    #[test]
    fn check_new_author_distinguishes_known() {
        let mut c = BatchCounters::from_known(["a@x".to_owned()]);
        let l = limits(); // max_new_authors_per_batch = 2
        assert!(matches!(c.check_new_author(l, "a@x"), QuotaCheck::Allow));
        assert!(matches!(c.check_new_author(l, "b@x"), QuotaCheck::Allow));
        assert!(matches!(c.check_new_author(l, "c@x"), QuotaCheck::Allow));
        // 2 новых автора (b, c) исчерпали квоту; d запрещён.
        assert!(c.check_new_author(l, "d@x").denied_reason().is_some());
    }

    #[test]
    fn check_deferred_respects_db_total() {
        let mut c = BatchCounters::default();
        let l = limits(); // max_deferred_total = 3, max_deferred_per_origin = 2
        // БД уже на пределе → запрет сразу.
        assert!(c.check_deferred(l, 10, "a@x").denied_reason().is_some());
        // per_origin = 2: два разрешения, третий запрет.
        assert!(matches!(c.check_deferred(l, 0, "a@x"), QuotaCheck::Allow));
        assert!(matches!(c.check_deferred(l, 0, "a@x"), QuotaCheck::Allow));
        assert!(c.check_deferred(l, 0, "a@x").denied_reason().is_some());
    }

    #[test]
    fn check_auto_response_increments() {
        let mut c = BatchCounters::default();
        let l = limits();
        assert!(matches!(c.check_auto_response(l), QuotaCheck::Allow));
        assert!(c.check_auto_response(l).denied_reason().is_some());
    }
}

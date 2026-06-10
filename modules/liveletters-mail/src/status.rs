use liveletters_mime::ReceivedEmail;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendStatus {
    Sent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchStatus {
    Fetched { message_count: usize },
    NoNewMessages,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailboxCursor {
    last_seen_uid: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchBatch {
    emails: Vec<ReceivedEmail>,
    next_cursor: MailboxCursor,
    status: FetchStatus,
}

impl MailboxCursor {
    pub fn start() -> Self {
        Self {
            last_seen_uid: None,
        }
    }

    pub fn from_last_seen_uid(last_seen_uid: u64) -> Self {
        Self {
            last_seen_uid: Some(last_seen_uid),
        }
    }

    /// Создаёт курсор, который "видел" все UID < since_uid. Это
    /// заставляет `fetch_new` начать с `since_uid` (потому что
    /// `start_uid = last_seen_uid + 1` в imap.rs:90). Используется
    /// при первом запуске с `initial_lookback_days` и при backfill.
    pub fn start_with_since_uid(since_uid: u64) -> Self {
        Self {
            last_seen_uid: Some(since_uid.saturating_sub(1)),
        }
    }

    pub fn last_seen_uid(&self) -> Option<u64> {
        self.last_seen_uid
    }

    pub fn advance_to(&self, uid: u64) -> Self {
        Self {
            last_seen_uid: Some(uid),
        }
    }
}

impl FetchBatch {
    pub fn new(emails: Vec<ReceivedEmail>, next_cursor: MailboxCursor) -> Self {
        let status = if emails.is_empty() {
            FetchStatus::NoNewMessages
        } else {
            FetchStatus::Fetched {
                message_count: emails.len(),
            }
        };

        Self {
            emails,
            next_cursor,
            status,
        }
    }

    pub fn emails(&self) -> &[ReceivedEmail] {
        &self.emails
    }

    pub fn into_emails(self) -> Vec<ReceivedEmail> {
        self.emails
    }

    pub fn next_cursor(&self) -> &MailboxCursor {
        &self.next_cursor
    }

    pub fn status(&self) -> &FetchStatus {
        &self.status
    }
}

#[cfg(test)]
mod tests {
    use super::MailboxCursor;

    #[test]
    fn start_with_since_uid_anchors_cursor_below_since() {
        // start_with_since_uid(100) — означает "все UID < 100 уже
        // учтены, start_uid = 100". После такого курсора fetch_new
        // начнёт с start_uid = last_seen_uid + 1 = 100.
        let cursor = MailboxCursor::start_with_since_uid(100);
        assert_eq!(cursor.last_seen_uid(), Some(99));
    }

    #[test]
    fn start_with_since_uid_zero_means_start_from_uid_one() {
        // since_uid = 0 — start_uid = max(1, 0) = 1, невозможно
        // уйти ниже. saturating_sub(1) даёт 0, а не подчёркивание.
        let cursor = MailboxCursor::start_with_since_uid(0);
        assert_eq!(cursor.last_seen_uid(), Some(0));
    }

    #[test]
    fn from_last_seen_uid_and_start_with_since_uid_are_compatible() {
        // Оба конструктора должны быть взаимозаменяемы для одного
        // и того же значения: from_last_seen_uid(N) даёт start_uid = N+1,
        // а start_with_since_uid(N+1) даёт start_uid = N+1.
        let a = MailboxCursor::from_last_seen_uid(10);
        let b = MailboxCursor::start_with_since_uid(11);
        assert_eq!(a.last_seen_uid(), Some(10));
        assert_eq!(b.last_seen_uid(), Some(10));
    }
}

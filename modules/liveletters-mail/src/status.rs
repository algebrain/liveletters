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
    emails: Vec<crate::ReceivedEmail>,
    next_cursor: MailboxCursor,
    status: FetchStatus,
}

impl MailboxCursor {
    pub fn start() -> Self {
        Self { last_seen_uid: None }
    }

    pub fn from_last_seen_uid(last_seen_uid: u64) -> Self {
        Self {
            last_seen_uid: Some(last_seen_uid),
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
    pub fn new(emails: Vec<crate::ReceivedEmail>, next_cursor: MailboxCursor) -> Self {
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

    pub fn emails(&self) -> &[crate::ReceivedEmail] {
        &self.emails
    }

    pub fn into_emails(self) -> Vec<crate::ReceivedEmail> {
        self.emails
    }

    pub fn next_cursor(&self) -> &MailboxCursor {
        &self.next_cursor
    }

    pub fn status(&self) -> &FetchStatus {
        &self.status
    }
}

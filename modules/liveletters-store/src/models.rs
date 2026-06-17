#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostRecord {
    pub post_id: String,
    pub resource_email: String,
    pub author_email: String,
    pub created_at: u64,
    pub body: String,
    pub visibility: String,
    pub hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentRecord {
    pub comment_id: String,
    pub post_id: String,
    pub parent_comment_id: Option<String>,
    pub author_email: String,
    pub created_at: u64,
    pub body: String,
    pub visibility: String,
    pub hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutboxDelivery {
    Direct(Vec<String>),
    ResourceSubscribers,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxRecord {
    pub event_id: String,
    pub event_type: String,
    pub author_email: String,
    pub resource_email: Option<String>,
    pub delivery: OutboxDelivery,
    pub message_body: String,
    pub message_id: Option<String>,
    pub subject: Option<String>,
    /// Локализованное тело письма, отдельное от `message_body`
    /// (где хранится JSON `ProtocolMessage`). Используется при сборке
    /// `text/plain` под-части в `liveletters-mime::build_protocol_email`.
    /// В JSON-поле `ProtocolMessage.human_readable_body` намеренно
    /// не сериализуется, чтобы избежать дублирования в wire-формате.
    pub human_readable_body: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawMessageRecord {
    pub message_id: String,
    pub raw_message: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawEventRecord {
    pub event_id: String,
    pub event_type: String,
    pub resource_id: String,
    pub payload_json: String,
    pub apply_status: String,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeferredEventRecord {
    pub event_id: String,
    pub event_type: String,
    pub reason: String,
    pub payload_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserSettingsRecord {
    pub profile_id: String,
    pub author_email: String,
    pub avatar_url: Option<String>,
    pub language: String,
    pub setup_completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorRecord {
    pub email: String,
    pub nickname: String,
    pub source: String,
    pub first_seen_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailSettingsRecord {
    pub profile_id: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_security: String,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_hello_domain: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub imap_security: String,
    pub imap_username: String,
    pub imap_password: String,
    pub imap_mailbox: String,
    /// Сколько суток назад от текущего момента заглядывать при
    /// самом первом sync. По умолчанию 1 (только сегодняшние письма).
    /// 0 = "с самого начала".
    pub initial_lookback_days: u32,
}

impl Default for MailSettingsRecord {
    fn default() -> Self {
        Self {
            profile_id: String::new(),
            smtp_host: String::new(),
            smtp_port: 0,
            smtp_security: String::new(),
            smtp_username: String::new(),
            smtp_password: String::new(),
            smtp_hello_domain: String::new(),
            imap_host: String::new(),
            imap_port: 0,
            imap_security: String::new(),
            imap_username: String::new(),
            imap_password: String::new(),
            imap_mailbox: String::new(),
            initial_lookback_days: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionRecord {
    pub resource_email: String,
    pub subscriber_email: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingSubscriptionRecord {
    pub profile_id: String,
    pub resource_email: String,
    pub requested_at: u64,
    pub last_attempt_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BounceRecord {
    pub original_message_id: String,
    pub event_id: Option<String>,
    pub final_recipient_email: Option<String>,
    pub status_code: Option<String>,
    pub diagnostic_code: Option<String>,
    pub received_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostRecord {
    pub post_id: String,
    pub resource_id: String,
    pub author_id: String,
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
    pub author_id: String,
    pub created_at: u64,
    pub body: String,
    pub visibility: String,
    pub hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxRecord {
    pub event_id: String,
    pub event_type: String,
    pub resource_id: String,
    pub message_body: String,
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
    pub nickname: String,
    pub email_address: String,
    pub avatar_url: Option<String>,
    pub setup_completed: bool,
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
}

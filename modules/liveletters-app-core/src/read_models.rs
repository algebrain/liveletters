#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostSummary {
    pub post_id: String,
    pub resource_id: String,
    pub author_id: String,
    pub created_at: u64,
    pub body: String,
    pub visibility: String,
    pub hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentSummary {
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
pub struct OutboxEntry {
    pub event_id: String,
    pub event_type: String,
    pub resource_id: String,
    pub delivery: liveletters_store::OutboxDelivery,
    pub message_body: String,
    pub subject: Option<String>,
    pub message_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentUserPosts {
    posts: Vec<PostSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostThread {
    post: PostSummary,
    comments: Vec<CommentSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingOutbox {
    entries: Vec<OutboxEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeferredReprocessingSummary {
    pub applied: usize,
    pub replayed: usize,
    pub unauthorized: usize,
    pub invalid: usize,
    pub filtered: usize,
    pub still_deferred: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapState {
    pub setup_completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSettings {
    pub nickname: String,
    pub email_address: String,
    pub avatar_url: Option<String>,
    pub language: String,
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
    pub initial_lookback_days: u32,
    pub setup_completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriberEntry {
    pub subscriber_delivery_address: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionsList {
    pub owned_subscribers: Vec<SubscriberEntry>,
    pub subscribed_addresses: Vec<String>,
}

impl SubscriptionsList {
    pub fn new(owned_subscribers: Vec<SubscriberEntry>, subscribed_addresses: Vec<String>) -> Self {
        Self {
            owned_subscribers,
            subscribed_addresses,
        }
    }

    pub fn owned_subscribers(&self) -> &[SubscriberEntry] {
        &self.owned_subscribers
    }

    pub fn subscribed_addresses(&self) -> &[String] {
        &self.subscribed_addresses
    }
}

impl CurrentUserPosts {
    pub fn new(posts: Vec<PostSummary>) -> Self {
        Self { posts }
    }

    pub fn posts(&self) -> &[PostSummary] {
        &self.posts
    }
}

impl PostThread {
    pub fn new(post: PostSummary, comments: Vec<CommentSummary>) -> Self {
        Self { post, comments }
    }

    pub fn post(&self) -> &PostSummary {
        &self.post
    }

    pub fn comments(&self) -> &[CommentSummary] {
        &self.comments
    }
}

impl PendingOutbox {
    pub fn new(entries: Vec<OutboxEntry>) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> &[OutboxEntry] {
        &self.entries
    }
}

impl DeferredReprocessingSummary {
    pub fn new(
        applied: usize,
        replayed: usize,
        unauthorized: usize,
        invalid: usize,
        filtered: usize,
        still_deferred: usize,
    ) -> Self {
        Self {
            applied,
            replayed,
            unauthorized,
            invalid,
            filtered,
            still_deferred,
        }
    }
}

impl BootstrapState {
    pub fn new(setup_completed: bool) -> Self {
        Self { setup_completed }
    }
}

impl AppSettings {
    pub fn empty() -> Self {
        Self {
            nickname: String::new(),
            email_address: String::new(),
            avatar_url: None,
            language: liveletters_i18n::detect_system_locale().as_str().to_owned(),
            smtp_host: String::new(),
            smtp_port: 587,
            smtp_security: "starttls".to_owned(),
            smtp_username: String::new(),
            smtp_password: String::new(),
            smtp_hello_domain: String::new(),
            imap_host: String::new(),
            imap_port: 143,
            imap_security: "starttls".to_owned(),
            imap_username: String::new(),
            imap_password: String::new(),
            imap_mailbox: "INBOX".to_owned(),
            initial_lookback_days: 1,
            setup_completed: false,
        }
    }
}

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
    pub message_body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeFeed {
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
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_hello_domain: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub imap_username: String,
    pub imap_password: String,
    pub imap_mailbox: String,
    pub setup_completed: bool,
}

impl HomeFeed {
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
        still_deferred: usize,
    ) -> Self {
        Self {
            applied,
            replayed,
            unauthorized,
            invalid,
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
            smtp_host: String::new(),
            smtp_port: 587,
            smtp_username: String::new(),
            smtp_password: String::new(),
            smtp_hello_domain: String::new(),
            imap_host: String::new(),
            imap_port: 143,
            imap_username: String::new(),
            imap_password: String::new(),
            imap_mailbox: "INBOX".to_owned(),
            setup_completed: false,
        }
    }
}

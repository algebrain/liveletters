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

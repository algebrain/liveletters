#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatePostRequest<'a> {
    pub post_id: &'a str,
    pub resource_id: &'a str,
    pub author_id: &'a str,
    pub created_at: u64,
    pub body: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncStatusDto {
    pub status: String,
    pub applied_messages: usize,
    pub duplicate_messages: usize,
    pub malformed_messages: usize,
    pub deferred_events: usize,
    pub pending_outbox: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncomingFailureDto {
    pub message_id: String,
    pub status: String,
    pub preview: String,
}

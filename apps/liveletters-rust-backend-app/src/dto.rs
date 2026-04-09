use liveletters_app_core::{HomeFeed, PostThread};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatePostRequest<'a> {
    pub post_id: &'a str,
    pub resource_id: &'a str,
    pub author_id: &'a str,
    pub created_at: u64,
    pub body: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreatePostCommandRequest {
    pub post_id: String,
    pub resource_id: String,
    pub author_id: String,
    pub created_at: u64,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontendErrorLogRequest {
    pub kind: String,
    pub message: String,
    pub stack: Option<String>,
    pub source: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SyncStatusDto {
    pub status: String,
    pub applied_messages: usize,
    pub duplicate_messages: usize,
    pub replayed_messages: usize,
    pub unauthorized_messages: usize,
    pub invalid_messages: usize,
    pub malformed_messages: usize,
    pub deferred_events: usize,
    pub pending_outbox: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IncomingFailureDto {
    pub message_id: String,
    pub status: String,
    pub preview: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EventFailureDto {
    pub event_id: String,
    pub event_type: String,
    pub resource_id: String,
    pub apply_status: String,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BootstrapStateDto {
    pub setup_completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SettingsDto {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveSettingsRequest<'a> {
    pub nickname: &'a str,
    pub email_address: &'a str,
    pub avatar_url: Option<&'a str>,
    pub smtp_host: &'a str,
    pub smtp_port: u16,
    pub smtp_username: &'a str,
    pub smtp_password: &'a str,
    pub smtp_hello_domain: &'a str,
    pub imap_host: &'a str,
    pub imap_port: u16,
    pub imap_username: &'a str,
    pub imap_password: &'a str,
    pub imap_mailbox: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SaveSettingsCommandRequest {
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostSummaryDto {
    pub post_id: String,
    pub resource_id: String,
    pub author_id: String,
    pub created_at: u64,
    pub body: String,
    pub visibility: String,
    pub hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommentSummaryDto {
    pub comment_id: String,
    pub post_id: String,
    pub parent_comment_id: Option<String>,
    pub author_id: String,
    pub created_at: u64,
    pub body: String,
    pub visibility: String,
    pub hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HomeFeedDto {
    pub posts: Vec<PostSummaryDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostThreadDto {
    pub post: PostSummaryDto,
    pub comments: Vec<CommentSummaryDto>,
}

impl From<HomeFeed> for HomeFeedDto {
    fn from(value: HomeFeed) -> Self {
        Self {
            posts: value.posts().iter().cloned().map(PostSummaryDto::from).collect(),
        }
    }
}

impl From<PostThread> for PostThreadDto {
    fn from(value: PostThread) -> Self {
        Self {
            post: value.post().clone().into(),
            comments: value
                .comments()
                .iter()
                .cloned()
                .map(CommentSummaryDto::from)
                .collect(),
        }
    }
}

impl From<liveletters_app_core::PostSummary> for PostSummaryDto {
    fn from(value: liveletters_app_core::PostSummary) -> Self {
        Self {
            post_id: value.post_id,
            resource_id: value.resource_id,
            author_id: value.author_id,
            created_at: value.created_at,
            body: value.body,
            visibility: value.visibility,
            hidden: value.hidden,
        }
    }
}

impl From<liveletters_app_core::CommentSummary> for CommentSummaryDto {
    fn from(value: liveletters_app_core::CommentSummary) -> Self {
        Self {
            comment_id: value.comment_id,
            post_id: value.post_id,
            parent_comment_id: value.parent_comment_id,
            author_id: value.author_id,
            created_at: value.created_at,
            body: value.body,
            visibility: value.visibility,
            hidden: value.hidden,
        }
    }
}

impl From<liveletters_app_core::BootstrapState> for BootstrapStateDto {
    fn from(value: liveletters_app_core::BootstrapState) -> Self {
        Self {
            setup_completed: value.setup_completed,
        }
    }
}

impl From<liveletters_app_core::AppSettings> for SettingsDto {
    fn from(value: liveletters_app_core::AppSettings) -> Self {
        Self {
            nickname: value.nickname,
            email_address: value.email_address,
            avatar_url: value.avatar_url,
            smtp_host: value.smtp_host,
            smtp_port: value.smtp_port,
            smtp_username: value.smtp_username,
            smtp_password: value.smtp_password,
            smtp_hello_domain: value.smtp_hello_domain,
            imap_host: value.imap_host,
            imap_port: value.imap_port,
            imap_username: value.imap_username,
            imap_password: value.imap_password,
            imap_mailbox: value.imap_mailbox,
            setup_completed: value.setup_completed,
        }
    }
}

mod commands;
mod errors;
mod queries;
mod read_models;
mod service;

pub use commands::{
    CreateCommentCommand, CreateCommentResult, CreatePostCommand, CreatePostResult,
    EditCommentCommand, EditCommentResult, HidePostCommand, HidePostResult,
    ReprocessDeferredEventsCommand, ReprocessDeferredEventsResult,
};
pub use errors::AppCoreError;
pub use queries::{GetHomeFeedQuery, GetPendingOutboxQuery, GetPostThreadQuery};
pub use read_models::{
    CommentSummary, DeferredReprocessingSummary, HomeFeed, OutboxEntry, PendingOutbox, PostSummary,
    PostThread,
};
pub use service::AppCore;

pub fn crate_name() -> &'static str {
    "liveletters-app-core"
}

fn decode_visibility_name(value: &str) -> String {
    value.to_owned()
}

#[cfg(test)]
mod tests {
    use super::crate_name;

    #[test]
    fn exposes_crate_name() {
        assert_eq!(crate_name(), "liveletters-app-core");
    }
}

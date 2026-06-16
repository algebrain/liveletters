mod commands;
mod errors;
mod i18n_strings;
mod ids;
mod queries;
mod read_models;
mod service;

pub use commands::{
    CreateCommentCommand, CreateCommentFromIdentityCommand, CreateCommentResult, CreatePostCommand,
    CreatePostFromIdentityCommand, CreatePostResult, EditCommentCommand, EditCommentResult,
    HidePostCommand, HidePostResult, Identity, ReprocessDeferredEventsCommand,
    ReprocessDeferredEventsResult, SaveSettingsCommand, SaveSettingsResult, SubscribeCommand,
    SubscribeResult, UnsubscribeCommand, UnsubscribeResult, create_comment_from_identity,
    create_post_from_identity, subscribe, unsubscribe,
};
pub use errors::AppCoreError;
pub use i18n_strings::{
    SubjectAndBody, comment_created, comment_created_redistribute, comment_edited, locale_for,
    post_created, post_hidden, subscription_confirmed_accepted, subscription_confirmed_declined,
    subscription_requested, subscription_revoked,
};
pub use ids::{new_comment_id, new_post_id, unix_millis_now};
pub use liveletters_domain::Visibility;
pub use queries::{
    GetBootstrapStateQuery, GetCurrentUserPostsQuery, GetPendingOutboxQuery, GetPostThreadQuery,
    GetSettingsQuery, ListSubscriptionsQuery, get_bootstrap_state, get_current_user_posts,
    get_pending_outbox, get_post_thread, get_settings, list_subscriptions,
};
pub use read_models::{
    AppSettings, BootstrapState, CommentSummary, CurrentUserPosts, DeferredReprocessingSummary,
    OutboxEntry, PendingOutbox, PostSummary, PostThread, SubscriberEntry, SubscriptionsList,
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

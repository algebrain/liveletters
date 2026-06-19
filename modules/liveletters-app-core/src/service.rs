use liveletters_store::Store;

use crate::{
    AppCoreError, AppSettings, BootstrapState, CreateCommentCommand,
    CreateCommentFromIdentityCommand, CreateCommentResult, CreatePostCommand,
    CreatePostFromIdentityCommand, CreatePostResult, CurrentUserPosts, EditCommentCommand,
    EditCommentResult, FriendCommand, FriendResult, GetBootstrapStateQuery,
    GetCurrentUserPostsQuery, GetPendingOutboxQuery, GetPostThreadQuery, GetSettingsQuery,
    HidePostCommand, HidePostResult, ListSubscriptionsQuery, PendingOutbox, PostThread,
    ReprocessDeferredEventsCommand, ReprocessDeferredEventsResult, SaveSettingsCommand,
    SaveSettingsResult, SubscribeCommand, SubscribeResult, SubscriptionsList, UnsubscribeCommand,
    UnsubscribeResult, commands, queries,
};

pub struct AppCore<'a> {
    store: &'a Store,
}

impl<'a> AppCore<'a> {
    pub fn new(store: &'a Store) -> Self {
        Self { store }
    }

    pub fn create_post(
        &self,
        command: CreatePostCommand<'_>,
    ) -> Result<CreatePostResult, AppCoreError> {
        commands::create_post(self.store, command)
    }

    pub fn create_post_from_identity(
        &self,
        command: CreatePostFromIdentityCommand<'_>,
    ) -> Result<CreatePostResult, AppCoreError> {
        commands::create_post_from_identity(self.store, command)
    }

    pub fn create_comment_from_identity(
        &self,
        command: CreateCommentFromIdentityCommand<'_>,
    ) -> Result<CreateCommentResult, AppCoreError> {
        commands::create_comment_from_identity(self.store, command)
    }

    pub fn create_comment(
        &self,
        command: CreateCommentCommand<'_>,
    ) -> Result<CreateCommentResult, AppCoreError> {
        commands::create_comment(self.store, command)
    }

    pub fn get_current_user_posts(
        &self,
        query: GetCurrentUserPostsQuery<'_>,
    ) -> Result<CurrentUserPosts, AppCoreError> {
        queries::get_current_user_posts(self.store, query)
    }

    pub fn hide_post(&self, command: HidePostCommand<'_>) -> Result<HidePostResult, AppCoreError> {
        commands::hide_post(self.store, command)
    }

    pub fn edit_comment(
        &self,
        command: EditCommentCommand<'_>,
    ) -> Result<EditCommentResult, AppCoreError> {
        commands::edit_comment(self.store, command)
    }

    pub fn get_post_thread(
        &self,
        query: GetPostThreadQuery<'_>,
    ) -> Result<PostThread, AppCoreError> {
        queries::get_post_thread(self.store, query)
    }

    pub fn get_pending_outbox(
        &self,
        query: GetPendingOutboxQuery,
    ) -> Result<PendingOutbox, AppCoreError> {
        queries::get_pending_outbox(self.store, query)
    }

    pub fn get_bootstrap_state(
        &self,
        query: GetBootstrapStateQuery,
    ) -> Result<BootstrapState, AppCoreError> {
        queries::get_bootstrap_state(self.store, query)
    }

    pub fn get_settings(&self, query: GetSettingsQuery) -> Result<AppSettings, AppCoreError> {
        queries::get_settings(self.store, query)
    }

    pub fn reprocess_deferred_events(
        &self,
        command: ReprocessDeferredEventsCommand,
    ) -> Result<ReprocessDeferredEventsResult, AppCoreError> {
        commands::reprocess_deferred_events(self.store, command)
    }

    pub fn save_settings(
        &self,
        command: SaveSettingsCommand<'_>,
    ) -> Result<SaveSettingsResult, AppCoreError> {
        commands::save_settings(self.store, command)
    }

    pub fn subscribe(
        &self,
        command: SubscribeCommand<'_>,
    ) -> Result<SubscribeResult, AppCoreError> {
        commands::subscribe(self.store, command)
    }

    pub fn unsubscribe(
        &self,
        command: UnsubscribeCommand<'_>,
    ) -> Result<UnsubscribeResult, AppCoreError> {
        commands::unsubscribe(self.store, command)
    }

    pub fn friend(&self, command: FriendCommand<'_>) -> Result<FriendResult, AppCoreError> {
        commands::friend(self.store, command)
    }

    pub fn complete_pending_friend_after_subscription(
        &self,
        profile_id: &str,
        subscribed_resource_address: &str,
    ) -> Result<Option<FriendResult>, AppCoreError> {
        commands::complete_pending_friend_after_subscription(
            self.store,
            profile_id,
            subscribed_resource_address,
        )
    }

    pub fn list_subscriptions(
        &self,
        query: ListSubscriptionsQuery<'_>,
    ) -> Result<SubscriptionsList, AppCoreError> {
        queries::list_subscriptions(self.store, query)
    }
}

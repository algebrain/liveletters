use liveletters_store::Store;

use crate::{
    AppCoreError, AppSettings, BootstrapState, CreateCommentCommand, CreateCommentResult,
    CreatePostCommand, CreatePostResult, EditCommentCommand, EditCommentResult,
    GetBootstrapStateQuery, GetHomeFeedQuery, GetPendingOutboxQuery, GetPostThreadQuery,
    GetSettingsQuery, HidePostCommand, HidePostResult, HomeFeed, PendingOutbox, PostThread,
    ReprocessDeferredEventsCommand, ReprocessDeferredEventsResult, SaveSettingsCommand,
    SaveSettingsResult, commands, queries,
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
        commands::create_post(&self.store, command)
    }

    pub fn create_comment(
        &self,
        command: CreateCommentCommand<'_>,
    ) -> Result<CreateCommentResult, AppCoreError> {
        commands::create_comment(&self.store, command)
    }

    pub fn get_home_feed(&self, query: GetHomeFeedQuery) -> Result<HomeFeed, AppCoreError> {
        queries::get_home_feed(&self.store, query)
    }

    pub fn hide_post(&self, command: HidePostCommand<'_>) -> Result<HidePostResult, AppCoreError> {
        commands::hide_post(&self.store, command)
    }

    pub fn edit_comment(
        &self,
        command: EditCommentCommand<'_>,
    ) -> Result<EditCommentResult, AppCoreError> {
        commands::edit_comment(&self.store, command)
    }

    pub fn get_post_thread(
        &self,
        query: GetPostThreadQuery<'_>,
    ) -> Result<PostThread, AppCoreError> {
        queries::get_post_thread(&self.store, query)
    }

    pub fn get_pending_outbox(
        &self,
        query: GetPendingOutboxQuery,
    ) -> Result<PendingOutbox, AppCoreError> {
        queries::get_pending_outbox(&self.store, query)
    }

    pub fn get_bootstrap_state(
        &self,
        query: GetBootstrapStateQuery,
    ) -> Result<BootstrapState, AppCoreError> {
        queries::get_bootstrap_state(&self.store, query)
    }

    pub fn get_settings(&self, query: GetSettingsQuery) -> Result<AppSettings, AppCoreError> {
        queries::get_settings(&self.store, query)
    }

    pub fn reprocess_deferred_events(
        &self,
        command: ReprocessDeferredEventsCommand,
    ) -> Result<ReprocessDeferredEventsResult, AppCoreError> {
        commands::reprocess_deferred_events(&self.store, command)
    }

    pub fn save_settings(
        &self,
        command: SaveSettingsCommand<'_>,
    ) -> Result<SaveSettingsResult, AppCoreError> {
        commands::save_settings(&self.store, command)
    }
}

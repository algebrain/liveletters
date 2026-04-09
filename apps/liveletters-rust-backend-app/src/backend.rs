use liveletters_app_core::{
    AppCore, CreatePostCommand, GetHomeFeedQuery, GetPostThreadQuery, HomeFeed, PostThread,
};
use liveletters_diagnostics::{DiagnosticsReader, HealthStatus};
use liveletters_store::Store;

use crate::{BackendError, CreatePostRequest, EventFailureDto, IncomingFailureDto, SyncStatusDto};

pub struct BackendApp {
    store: Store,
}

impl BackendApp {
    pub fn from_store(store: Store) -> Self {
        Self { store }
    }

    pub fn open_in_memory() -> Result<Self, BackendError> {
        Ok(Self::from_store(Store::open_in_memory()?))
    }

    pub fn open_default() -> Result<Self, BackendError> {
        Ok(Self::from_store(Store::open_default()?))
    }

    pub fn create_post(&self, request: CreatePostRequest<'_>) -> Result<(), BackendError> {
        let app_core = AppCore::new(&self.store);
        app_core.create_post(CreatePostCommand {
            post_id: request.post_id,
            resource_id: request.resource_id,
            author_id: request.author_id,
            created_at: request.created_at,
            body: request.body,
        })?;
        Ok(())
    }

    pub fn get_home_feed(&self) -> Result<HomeFeed, BackendError> {
        let app_core = AppCore::new(&self.store);
        Ok(app_core.get_home_feed(GetHomeFeedQuery)?)
    }

    pub fn get_post_thread(&self, post_id: &str) -> Result<PostThread, BackendError> {
        let app_core = AppCore::new(&self.store);
        Ok(app_core.get_post_thread(GetPostThreadQuery { post_id })?)
    }

    pub fn get_sync_status(&self) -> Result<SyncStatusDto, BackendError> {
        let diagnostics = DiagnosticsReader::new(&self.store).build_snapshot()?;
        let health = diagnostics.sync_health();

        Ok(SyncStatusDto {
            status: match health.status {
                HealthStatus::Healthy => "healthy",
                HealthStatus::Degraded => "degraded",
            }
            .to_owned(),
            applied_messages: health.applied_messages,
            duplicate_messages: health.duplicate_messages,
            replayed_messages: health.replayed_messages,
            unauthorized_messages: health.unauthorized_messages,
            invalid_messages: health.invalid_messages,
            malformed_messages: health.malformed_messages,
            deferred_events: health.deferred_events,
            pending_outbox: health.pending_outbox,
        })
    }

    pub fn list_incoming_failures(&self) -> Result<Vec<IncomingFailureDto>, BackendError> {
        let diagnostics = DiagnosticsReader::new(&self.store).build_snapshot()?;

        Ok(diagnostics
            .raw_messages()
            .iter()
            .filter(|message| message.status != "applied")
            .map(|message| IncomingFailureDto {
                message_id: message.message_id.clone(),
                status: message.status.clone(),
                preview: message.preview.clone(),
            })
            .collect())
    }

    pub fn list_event_failures(&self) -> Result<Vec<EventFailureDto>, BackendError> {
        let diagnostics = DiagnosticsReader::new(&self.store).build_snapshot()?;

        Ok(diagnostics
            .raw_events()
            .iter()
            .filter(|event| event.apply_status != "applied" && event.apply_status != "pending")
            .map(|event| EventFailureDto {
                event_id: event.event_id.clone(),
                event_type: event.event_type.clone(),
                resource_id: event.resource_id.clone(),
                apply_status: event.apply_status.clone(),
                failure_reason: event.failure_reason.clone(),
            })
            .collect())
    }
}

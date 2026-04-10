use serde::Serialize;

pub const FEED_UPDATED_EVENT: &str = "feed-updated";
pub const SYNC_STATUS_CHANGED_EVENT: &str = "sync-status-changed";
#[cfg(feature = "tauri-runtime")]
pub const FRONTEND_ERROR_EVENT: &str = "frontend-error";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FrontendEvent {
    pub reason: &'static str,
}

impl FrontendEvent {
    pub const fn feed_updated() -> Self {
        Self {
            reason: "feed-updated",
        }
    }

    pub const fn sync_status_changed() -> Self {
        Self {
            reason: "sync-status-changed",
        }
    }

    pub const fn app_booted() -> Self {
        Self {
            reason: "app-booted",
        }
    }

    pub fn name(&self) -> &'static str {
        match self.reason {
            "feed-updated" => FEED_UPDATED_EVENT,
            _ => SYNC_STATUS_CHANGED_EVENT,
        }
    }
}

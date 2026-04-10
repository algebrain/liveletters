mod backend;
mod dto;
mod errors;
mod events;
mod runtime_options;
#[cfg(feature = "tauri-runtime")]
mod bootstrap;
#[cfg(feature = "tauri-runtime")]
mod commands;
#[cfg(feature = "tauri-runtime")]
mod runtime;

pub use backend::BackendApp;
pub use dto::{
    BootstrapStateDto, CommentSummaryDto, CreatePostCommandRequest, CreatePostRequest,
    EventFailureDto, FrontendErrorLogRequest, HomeFeedDto, IncomingFailureDto, PostSummaryDto,
    PostThreadDto, SaveSettingsCommandRequest, SaveSettingsRequest, SettingsDto, SyncStatusDto,
};
pub use errors::{BackendError, CommandErrorDto};
pub use events::FrontendEvent;
pub use runtime_options::RuntimeOptions;
#[cfg(feature = "tauri-runtime")]
pub use bootstrap::build as build_tauri_app;
#[cfg(feature = "tauri-runtime")]
pub use runtime::{runtime_log_dir, runtime_log_dir_for_home, runtime_log_line, BackendState};

pub fn app_name() -> &'static str {
    "liveletters-rust-backend-app"
}

#[cfg(test)]
mod tests {
    use super::app_name;

    #[test]
    fn exposes_app_name() {
        assert_eq!(app_name(), "liveletters-rust-backend-app");
    }
}

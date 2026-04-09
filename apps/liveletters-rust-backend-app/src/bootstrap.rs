#![cfg(feature = "tauri-runtime")]

use tauri::{Builder, Emitter, Listener};

use crate::{
    commands,
    events::{FrontendEvent, FRONTEND_ERROR_EVENT},
    runtime::{append_runtime_log, runtime_log_line, BackendState},
    FrontendErrorLogRequest, BackendApp, BackendError,
};

pub fn build() -> Result<Builder<tauri::Wry>, BackendError> {
    // In constrained dev/sandbox environments, default HOME-backed storage may be unavailable.
    // Fall back to in-memory store to keep the runtime bridge bootable.
    let backend = BackendApp::open_default().or_else(|_error| BackendApp::open_in_memory())?;
    let state = BackendState::new(backend);

    Ok(tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::create_post,
            commands::get_home_feed,
            commands::get_post_thread,
            commands::get_sync_status,
            commands::list_incoming_failures,
            commands::list_event_failures,
            commands::log_frontend_error
        ])
        .setup(|app| {
            app.listen(FRONTEND_ERROR_EVENT, |_event| {
                let payload = _event.payload();
                match serde_json::from_str::<FrontendErrorLogRequest>(payload) {
                    Ok(request) => {
                        let line = runtime_log_line(
                            &request.kind,
                            &request.message,
                            request.stack.as_deref(),
                            request.source.as_deref(),
                            request.location.as_deref(),
                        );
                        let _ = append_runtime_log("frontend-errors.log", &line);
                    }
                    Err(error) => {
                        let line = runtime_log_line(
                            "frontend-error.invalid-payload",
                            &format!("failed to parse event payload: {error}"),
                            Some(payload),
                            None,
                            None,
                        );
                        let _ = append_runtime_log("frontend-errors.log", &line);
                    }
                }
            });
            app.emit(FrontendEvent::sync_status_changed().name(), FrontendEvent::app_booted())?;
            Ok(())
        }))
}

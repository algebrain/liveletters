#![cfg(feature = "tauri-runtime")]

use tauri::{AppHandle, State};

use crate::{
    runtime::{append_runtime_log, emit_frontend_event, runtime_log_line, BackendState},
    BootstrapStateDto, CommandErrorDto, CreatePostCommandRequest, CreatePostRequest,
    EventFailureDto, FrontendErrorLogRequest, FrontendEvent, HomeFeedDto, IncomingFailureDto,
    PostThreadDto, SaveSettingsCommandRequest, SaveSettingsRequest, SettingsDto, SyncStatusDto,
};

#[tauri::command]
pub fn create_post(
    app: AppHandle,
    state: State<'_, BackendState>,
    request: CreatePostCommandRequest,
) -> Result<(), CommandErrorDto> {
    state
        .with_backend(|backend| {
            backend.create_post(CreatePostRequest {
                post_id: &request.post_id,
                resource_id: &request.resource_id,
                author_id: &request.author_id,
                created_at: request.created_at,
                body: &request.body,
            })
        })
        .map_err(CommandErrorDto::from)?;

    emit_frontend_event(&app, FrontendEvent::feed_updated())?;
    emit_frontend_event(&app, FrontendEvent::sync_status_changed())?;

    Ok(())
}

#[tauri::command]
pub fn get_home_feed(state: State<'_, BackendState>) -> Result<HomeFeedDto, CommandErrorDto> {
    state
        .with_backend(|backend| backend.get_home_feed())
        .map(HomeFeedDto::from)
        .map_err(CommandErrorDto::from)
}

#[tauri::command]
pub fn get_bootstrap_state(
    state: State<'_, BackendState>,
) -> Result<BootstrapStateDto, CommandErrorDto> {
    state
        .with_backend(|backend| backend.get_bootstrap_state())
        .map_err(CommandErrorDto::from)
}

#[tauri::command]
pub fn get_settings(state: State<'_, BackendState>) -> Result<SettingsDto, CommandErrorDto> {
    state
        .with_backend(|backend| backend.get_settings())
        .map_err(CommandErrorDto::from)
}

#[tauri::command]
pub fn save_settings(
    state: State<'_, BackendState>,
    request: SaveSettingsCommandRequest,
) -> Result<SettingsDto, CommandErrorDto> {
    state
        .with_backend(|backend| {
            backend.save_settings(SaveSettingsRequest {
                nickname: &request.nickname,
                email_address: &request.email_address,
                avatar_url: request.avatar_url.as_deref(),
                smtp_host: &request.smtp_host,
                smtp_port: request.smtp_port,
                smtp_username: &request.smtp_username,
                smtp_password: &request.smtp_password,
                smtp_hello_domain: &request.smtp_hello_domain,
                imap_host: &request.imap_host,
                imap_port: request.imap_port,
                imap_username: &request.imap_username,
                imap_password: &request.imap_password,
                imap_mailbox: &request.imap_mailbox,
            })
        })
        .map_err(CommandErrorDto::from)
}

#[tauri::command]
pub fn get_post_thread(
    state: State<'_, BackendState>,
    post_id: String,
) -> Result<PostThreadDto, CommandErrorDto> {
    state
        .with_backend(|backend| backend.get_post_thread(&post_id))
        .map(PostThreadDto::from)
        .map_err(CommandErrorDto::from)
}

#[tauri::command]
pub fn get_sync_status(state: State<'_, BackendState>) -> Result<SyncStatusDto, CommandErrorDto> {
    state
        .with_backend(|backend| backend.get_sync_status())
        .map_err(CommandErrorDto::from)
}

#[tauri::command]
pub fn list_incoming_failures(
    state: State<'_, BackendState>,
) -> Result<Vec<IncomingFailureDto>, CommandErrorDto> {
    state
        .with_backend(|backend| backend.list_incoming_failures())
        .map_err(CommandErrorDto::from)
}

#[tauri::command]
pub fn list_event_failures(
    state: State<'_, BackendState>,
) -> Result<Vec<EventFailureDto>, CommandErrorDto> {
    state
        .with_backend(|backend| backend.list_event_failures())
        .map_err(CommandErrorDto::from)
}

#[tauri::command]
pub fn log_frontend_error(request: FrontendErrorLogRequest) -> Result<(), CommandErrorDto> {
    let line = runtime_log_line(
        &request.kind,
        &request.message,
        request.stack.as_deref(),
        request.source.as_deref(),
        request.location.as_deref(),
    );

    append_runtime_log("frontend-errors.log", &line)
}

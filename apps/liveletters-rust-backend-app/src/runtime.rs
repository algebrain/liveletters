#![cfg(feature = "tauri-runtime")]

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use tauri::{AppHandle, Emitter};

use crate::{BackendApp, BackendError, CommandErrorDto, FrontendEvent};

pub struct BackendState {
    backend: Mutex<BackendApp>,
}

impl BackendState {
    pub fn new(backend: BackendApp) -> Self {
        Self {
            backend: Mutex::new(backend),
        }
    }

    pub fn with_backend<T>(
        &self,
        f: impl FnOnce(&BackendApp) -> Result<T, BackendError>,
    ) -> Result<T, BackendError> {
        let guard = self
            .backend
            .lock()
            .expect("backend state mutex should not be poisoned");

        f(&guard)
    }
}

pub fn emit_frontend_event(
    emitter: &AppHandle,
    event: FrontendEvent,
) -> Result<(), CommandErrorDto> {
    emitter.emit(event.name(), event).map_err(CommandErrorDto::emit)
}

pub fn runtime_log_dir() -> PathBuf {
    std::env::var_os("LIVELETTERS_RUNTIME_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.docs/runtime-logs"))
}

pub fn append_runtime_log(filename: &str, line: &str) -> Result<(), CommandErrorDto> {
    let log_dir = runtime_log_dir();

    fs::create_dir_all(&log_dir).map_err(|error| {
        CommandErrorDto::internal(format!("failed to create runtime log directory: {error}"))
    })?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_dir.join(filename))
        .map_err(|error| CommandErrorDto::internal(format!("failed to open log file: {error}")))?;

    file.write_all(line.as_bytes())
        .map_err(|error| CommandErrorDto::internal(format!("failed to write log file: {error}")))?;

    Ok(())
}

pub fn runtime_log_line(
    kind: &str,
    message: &str,
    stack: Option<&str>,
    source: Option<&str>,
    location: Option<&str>,
) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    format!(
        "[ts={timestamp}] [kind={kind}] [message={}] [stack={}] [source={}] [location={}]\n",
        message.replace('\n', " "),
        stack.unwrap_or("").replace('\n', " "),
        source.unwrap_or(""),
        location.unwrap_or("")
    )
}

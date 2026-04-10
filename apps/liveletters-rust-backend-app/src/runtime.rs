#![cfg(feature = "tauri-runtime")]

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use liveletters_store::StorePaths;
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

pub fn runtime_log_dir_for_home(home_dir: impl AsRef<Path>) -> PathBuf {
    StorePaths::for_home_dir(home_dir).runtime_log_dir().to_path_buf()
}

pub fn runtime_log_dir() -> Option<PathBuf> {
    if !runtime_debug_logs_enabled() {
        return None;
    }

    std::env::var_os("LIVELETTERS_RUNTIME_LOG_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            StorePaths::from_environment()
                .ok()
                .map(|paths| paths.runtime_log_dir().to_path_buf())
        })
}

pub fn prepare_runtime_logs() -> Result<Option<PathBuf>, CommandErrorDto> {
    let Some(log_dir) = runtime_log_dir() else {
        return Ok(None);
    };

    rotate_runtime_logs(&log_dir).map_err(|error| {
        CommandErrorDto::internal(format!("failed to rotate runtime log directory: {error}"))
    })?;
    fs::create_dir_all(&log_dir).map_err(|error| {
        CommandErrorDto::internal(format!("failed to create runtime log directory: {error}"))
    })?;

    Ok(Some(log_dir))
}

pub fn append_runtime_log(filename: &str, line: &str) -> Result<(), CommandErrorDto> {
    let Some(log_dir) = runtime_log_dir() else {
        return Ok(());
    };

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

fn runtime_debug_logs_enabled() -> bool {
    std::env::var_os("LIVELETTERS_DEBUG_LOGS")
        .and_then(|value| value.into_string().ok())
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

fn rotate_runtime_logs(log_dir: &Path) -> Result<(), std::io::Error> {
    if !log_dir.exists() {
        return Ok(());
    }

    let archive_root = log_dir.join("archive");
    fs::create_dir_all(&archive_root)?;

    let mut to_move = Vec::new();
    for entry in fs::read_dir(log_dir)? {
        let entry = entry?;
        if entry.file_name() == "archive" {
            continue;
        }
        to_move.push(entry.path());
    }

    if to_move.is_empty() {
        return Ok(());
    }

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let archive_dir = archive_root.join(format!("{stamp}"));
    fs::create_dir_all(&archive_dir)?;

    for path in to_move {
        let Some(file_name) = path.file_name() else {
            continue;
        };
        fs::rename(&path, archive_dir.join(file_name))?;
    }

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

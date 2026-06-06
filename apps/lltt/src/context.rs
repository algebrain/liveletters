//! Построение [`CommandContext`] для всех команд `lltt`.
//!
//! [`CommandContext`]: ../../../modules/liveletters-output/src/context.rs

use std::path::{Path, PathBuf};

use liveletters_output::CommandContext;
use liveletters_store::resolve_data_dir_from_env;

/// Имя файла, в котором хранится имя текущего пользователя liveletters.
const CURRENT_USER_FILE: &str = "current-user";

const USERS_DIR: &str = "users";

/// Разрешает домашний каталог из `LIVELETTERS_HOME` или возвращает
/// `<user-home>/.liveletters/` (Unix — `$HOME`, Windows — `%USERPROFILE%`).
pub fn resolve_home() -> PathBuf {
    resolve_data_dir_from_env().unwrap_or_else(|| PathBuf::from("."))
}

/// Читает имя текущего пользователя liveletters из файла `<home>/current-user`.
///
/// Если файл отсутствует — возвращает [`ContextError::NoCurrentUser`].
/// Это сознательное поведение: пока ни одного пользователя не создано
/// (например, после ручного удаления файла), система неработоспособна.
pub fn resolve_current_user_name(home: &Path) -> Result<String, ContextError> {
    match liveletters_config::read_current_identity(home) {
        Ok(name) => Ok(name),
        Err(liveletters_config::ConfigError::NoCurrentUser(_)) => {
            Err(ContextError::NoCurrentUser(home.join(CURRENT_USER_FILE)))
        }
        Err(other) => Err(ContextError::Config(other)),
    }
}

/// Режим построения контекста для команд, которым не всегда нужен
/// выбранный текущий пользователь.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextMode {
    Init,
    AllowMissingCurrent,
    RequiresCurrent,
}

/// Собирает [`CommandContext`] из окружения.
pub fn build_context(mode: ContextMode) -> Result<CommandContext, ContextError> {
    let home = resolve_home();
    let identity_name = match mode {
        ContextMode::Init | ContextMode::AllowMissingCurrent => {
            resolve_current_user_name(&home).unwrap_or_default()
        }
        ContextMode::RequiresCurrent => resolve_current_user_name(&home)?,
    };
    let state_home = if identity_name.is_empty() {
        home.clone()
    } else {
        user_state_home(&home, &identity_name)
    };
    Ok(CommandContext {
        home,
        state_home,
        identity_name,
    })
}

pub fn user_state_home(home: &Path, identity_name: &str) -> PathBuf {
    home.join(USERS_DIR).join(identity_name)
}

#[derive(Debug)]
pub enum ContextError {
    /// Файл `<home>/current-user` отсутствует; система неработоспособна.
    NoCurrentUser(PathBuf),
    /// Прочая ошибка конфигурации (TOML, IO и т.п.).
    Config(liveletters_config::ConfigError),
}

impl std::fmt::Display for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoCurrentUser(path) => write!(
                f,
                "текущий пользователь liveletters не задан: файл `{}` отсутствует; создайте пользователя командой `lltt user init <имя>`, добавьте его командой `lltt user add <имя>`, затем выберите командой `lltt cu <имя>`",
                path.display()
            ),
            Self::Config(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for ContextError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NoCurrentUser(_) => None,
            Self::Config(err) => Some(err),
        }
    }
}

use std::{
    env,
    path::{Path, PathBuf},
};

use crate::StoreError;

const DEFAULT_HOME_SUFFIX: &str = ".liveletters";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorePaths {
    data_dir: PathBuf,
    database_path: PathBuf,
    runtime_log_dir: PathBuf,
    password_obfuscation_key_path: PathBuf,
}

impl StorePaths {
    pub fn for_home_dir(home_dir: impl AsRef<Path>) -> Self {
        let data_dir = home_dir.as_ref().to_path_buf();
        let database_path = data_dir.join("liveletters.sqlite3");
        let runtime_log_dir = data_dir.join("runtime-logs");
        let password_obfuscation_key_path = data_dir.join("mail-password-obfuscation.key");

        Self {
            data_dir,
            database_path,
            runtime_log_dir,
            password_obfuscation_key_path,
        }
    }

    pub fn from_environment() -> Result<Self, StoreError> {
        let home_dir = resolve_data_dir_from_env().ok_or(StoreError::MissingHomeDirectory)?;
        Ok(Self::for_home_dir(home_dir))
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn database_path(&self) -> &Path {
        &self.database_path
    }

    pub fn runtime_log_dir(&self) -> &Path {
        &self.runtime_log_dir
    }

    pub fn password_obfuscation_key_path(&self) -> &Path {
        &self.password_obfuscation_key_path
    }
}

/// Явные значения переменных окружения, которые [`resolve_data_dir`] принимает
/// на вход. Используется, чтобы тесты могли проверить логику разрешения пути
/// **без** мутации глобального окружения текущего процесса.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnvOverrides {
    pub liveletters_home: Option<PathBuf>,
    pub home: Option<PathBuf>,
    pub userprofile: Option<PathBuf>,
}

impl EnvOverrides {
    /// Считывает текущее окружение процесса. Обёртка над `std::env::var_os`.
    pub fn from_process() -> Self {
        Self {
            liveletters_home: env::var_os("LIVELETTERS_HOME").map(PathBuf::from),
            home: env::var_os("HOME").map(PathBuf::from),
            userprofile: env::var_os("USERPROFILE").map(PathBuf::from),
        }
    }
}

/// Чистая функция: разрешает каталог данных `lltt` по явным значениям окружения.
///
/// Приоритет:
/// 1. `env.liveletters_home` — используется как есть, без суффикса.
/// 2. `<user-home>/.liveletters/`, где `<user-home>` берётся из `env.home`
///    (Unix, MSYS/Cygwin/Git Bash) или `env.userprofile` (нативный Windows).
/// 3. `None` — если ничего не задано.
pub fn resolve_data_dir(env: &EnvOverrides) -> Option<PathBuf> {
    if let Some(value) = &env.liveletters_home {
        return Some(value.clone());
    }
    let user_home = env.home.clone().or_else(|| env.userprofile.clone())?;
    Some(user_home.join(DEFAULT_HOME_SUFFIX))
}

/// Разрешает каталог данных `lltt` из окружения текущего процесса.
///
/// Обёртка над [`resolve_data_dir`] + [`EnvOverrides::from_process`].
pub fn resolve_data_dir_from_env() -> Option<PathBuf> {
    resolve_data_dir(&EnvOverrides::from_process())
}

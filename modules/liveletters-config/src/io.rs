use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{ConfigError, GlobalConfig, IdentityConfig};

const GLOBAL_CONFIG_FILENAME: &str = "config.toml";
const IDENTITIES_DIRNAME: &str = "identities";
const CURRENT_USER_FILENAME: &str = "current-user";

pub fn load_global(home: &Path) -> Result<GlobalConfig, ConfigError> {
    let path = home.join(GLOBAL_CONFIG_FILENAME);
    if !path.exists() {
        return Ok(GlobalConfig::default());
    }
    let raw = fs::read_to_string(&path)?;
    let config = toml::from_str(&raw)?;
    Ok(config)
}

pub fn save_global(home: &Path, config: &GlobalConfig) -> Result<(), ConfigError> {
    let path = home.join(GLOBAL_CONFIG_FILENAME);
    let raw = toml::to_string_pretty(config)?;
    fs::write(path, raw)?;
    Ok(())
}

pub fn load_identity(home: &Path, name: &str) -> Result<IdentityConfig, ConfigError> {
    let path = identity_path(home, name);
    if !path.exists() {
        return Err(ConfigError::UnknownIdentity(name.to_owned()));
    }
    let raw = fs::read_to_string(&path)?;
    let config = toml::from_str(&raw)?;
    Ok(config)
}

pub fn save_identity(home: &Path, name: &str, cfg: &IdentityConfig) -> Result<(), ConfigError> {
    let dir = identities_dir(home);
    fs::create_dir_all(&dir)?;
    let path = identity_path(home, name);
    let raw = toml::to_string_pretty(cfg)?;
    fs::write(path, raw)?;
    Ok(())
}

pub fn list_identities(home: &Path) -> Result<Vec<String>, ConfigError> {
    let dir = identities_dir(home);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut names = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let Some(name) = file_name.to_str() else {
            continue;
        };
        let Some(stripped) = name.strip_suffix(".toml") else {
            continue;
        };
        names.push(stripped.to_owned());
    }
    names.sort();
    Ok(names)
}

fn identities_dir(home: &Path) -> PathBuf {
    home.join(IDENTITIES_DIRNAME)
}

fn identity_path(home: &Path, name: &str) -> PathBuf {
    identities_dir(home).join(format!("{name}.toml"))
}

/// Путь к файлу, в котором хранится имя текущего пользователя liveletters.
pub fn current_user_path(home: &Path) -> PathBuf {
    home.join(CURRENT_USER_FILENAME)
}

/// Читает имя текущего пользователя liveletters из `<home>/current-user`.
///
/// Возвращает `ConfigError::NoCurrentUser(path)`, если файл отсутствует.
/// Это сознательное поведение: пока ни одного пользователя не создано,
/// система неработоспособна (см. дизайн в `docs/idea-technical.md`).
pub fn read_current_identity(home: &Path) -> Result<String, ConfigError> {
    let path = current_user_path(home);
    if !path.exists() {
        return Err(ConfigError::NoCurrentUser(path));
    }
    let raw = fs::read_to_string(&path)?;
    Ok(raw.trim().to_owned())
}

/// Записывает имя текущего пользователя liveletters в `<home>/current-user`.
///
/// Используется командами `lltt cu <имя>` (переключение) и `lltt init`
/// (создание дефолтной записи `default`).
pub fn write_current_identity(home: &Path, name: &str) -> Result<(), ConfigError> {
    let path = current_user_path(home);
    fs::write(&path, name)?;
    Ok(())
}

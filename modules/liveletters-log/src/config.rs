use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Настройки журнала, хранятся в `GlobalConfig.log` (TOML-файл
/// `${LIVELETTERS_HOME}/config.toml`, секция `[log]`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogConfig {
    /// Куда писать журнал. По умолчанию `File`.
    #[serde(default)]
    pub destination: LogDestination,
    /// Минимальный уровень, ниже которого записи игнорируются.
    /// По умолчанию `Off` — журнал выключен.
    #[serde(default)]
    pub level: LogLevel,
    /// Максимальный размер текущего файла в байтах до ротации.
    /// `0` означает «использовать дефолт». Минимум — `1024`.
    #[serde(default = "default_max_size_bytes")]
    pub max_size_bytes: u64,
    /// Сколько архивных файлов хранить (`.1`, `.2`, …). `0` — дефолт.
    #[serde(default = "default_keep_files")]
    pub keep_files: u32,
    /// Разрешает запись тел писем и payload в журнал. По умолчанию `false`.
    /// Если включено, ответственность за нераспространение лога лежит на пользователе.
    #[serde(default)]
    pub include_bodies: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            destination: LogDestination::File,
            level: LogLevel::Off,
            max_size_bytes: default_max_size_bytes(),
            keep_files: default_keep_files(),
            include_bodies: false,
        }
    }
}

impl LogConfig {
    /// Обновляет одно поле по строковому ключу (без префикса `log.`).
    /// Возвращает `Err`, если ключ не распознан или значение невалидно.
    pub fn set_field(&mut self, key: &str, value: &str) -> Result<(), String> {
        match key {
            "destination" => {
                self.destination = LogDestination::from_str(value)?;
            }
            "level" => {
                self.level = LogLevel::from_str(value)?;
            }
            "max_size_bytes" => {
                self.max_size_bytes = parse_u64(value)?;
            }
            "keep_files" => {
                self.keep_files = parse_u32(value)?;
            }
            "include_bodies" => {
                self.include_bodies = parse_bool(value)?;
            }
            other => return Err(format!("неизвестное поле журнала: {other}")),
        }
        Ok(())
    }
}

fn default_max_size_bytes() -> u64 {
    5 * 1024 * 1024
}

fn default_keep_files() -> u32 {
    3
}

fn parse_u64(s: &str) -> Result<u64, String> {
    s.trim()
        .parse::<u64>()
        .map_err(|e| format!("некорректное целое: {e}"))
}

fn parse_u32(s: &str) -> Result<u32, String> {
    s.trim()
        .parse::<u32>()
        .map_err(|e| format!("некорректное целое: {e}"))
}

fn parse_bool(s: &str) -> Result<bool, String> {
    match s.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" | "да" => Ok(true),
        "false" | "0" | "no" | "off" | "нет" => Ok(false),
        other => Err(format!("ожидается true/false, получено: {other}")),
    }
}

/// Уровни логирования.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    #[default]
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    /// Числовое значение для быстрого сравнения в hot-path.
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Off => 0,
            Self::Error => 1,
            Self::Warn => 2,
            Self::Info => 3,
            Self::Debug => 4,
            Self::Trace => 5,
        }
    }

    /// Имя уровня, как оно пишется в строке лога.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Off => "off",
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        };
        f.write_str(s)
    }
}

impl FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "off" | "none" | "disabled" => Ok(Self::Off),
            "error" | "err" => Ok(Self::Error),
            "warn" | "warning" => Ok(Self::Warn),
            "info" => Ok(Self::Info),
            "debug" => Ok(Self::Debug),
            "trace" => Ok(Self::Trace),
            other => Err(format!("неизвестный уровень журнала: {other}")),
        }
    }
}

/// Куда писать журнал.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogDestination {
    /// Файл `${LIVELETTERS_HOME}/logs/liveletters.log`.
    #[default]
    File,
    /// Стандартный поток ошибок процесса.
    Stderr,
    /// Журнал никуда не пишется (используется, когда уровень `off`).
    None,
}

impl std::fmt::Display for LogDestination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::File => "file",
            Self::Stderr => "stderr",
            Self::None => "none",
        };
        f.write_str(s)
    }
}

impl FromStr for LogDestination {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "file" => Ok(Self::File),
            "stderr" => Ok(Self::Stderr),
            "none" | "off" | "disabled" => Ok(Self::None),
            other => Err(format!("неизвестное назначение журнала: {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_off_and_file() {
        let cfg = LogConfig::default();
        assert_eq!(cfg.destination, LogDestination::File);
        assert_eq!(cfg.level, LogLevel::Off);
        assert_eq!(cfg.max_size_bytes, 5 * 1024 * 1024);
        assert_eq!(cfg.keep_files, 3);
        assert!(!cfg.include_bodies);
    }

    #[test]
    fn set_field_accepts_known_keys() {
        let mut cfg = LogConfig::default();
        cfg.set_field("level", "info").unwrap();
        cfg.set_field("destination", "stderr").unwrap();
        cfg.set_field("max_size_bytes", "1048576").unwrap();
        cfg.set_field("keep_files", "5").unwrap();
        cfg.set_field("include_bodies", "true").unwrap();
        assert_eq!(cfg.level, LogLevel::Info);
        assert_eq!(cfg.destination, LogDestination::Stderr);
        assert_eq!(cfg.max_size_bytes, 1_048_576);
        assert_eq!(cfg.keep_files, 5);
        assert!(cfg.include_bodies);
    }

    #[test]
    fn set_field_rejects_unknown_key() {
        let mut cfg = LogConfig::default();
        assert!(cfg.set_field("bogus", "x").is_err());
    }

    #[test]
    fn set_field_rejects_bad_value() {
        let mut cfg = LogConfig::default();
        assert!(cfg.set_field("level", "loud").is_err());
        assert!(cfg.set_field("include_bodies", "maybe").is_err());
        assert!(cfg.set_field("keep_files", "abc").is_err());
    }
}

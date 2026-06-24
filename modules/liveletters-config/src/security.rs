//! Per-user настройки безопасности: `users/<name>/config.toml`.
//!
//! Файл создаётся один раз при `lltt user add` из кодовых defaults
//! ([`SecurityConfig::default_toml`]) и читается командным слоем при запуске
//! sync/инбокса. Файл намеренно не документирован и не редактируется через
//! `lltt settings`: правки пользователем возможны вручную, на свой страх и
//! риск.
//!
//! Поведение при чтении — «переопределение»: каждое поле подсекции несёт
//! собственный serde-default. Отсутствующий в файле ключ заменяется кодовым
//! значением; заданный — уважается. Таким образом, частичное переопределение
//! вида
//!
//! ```toml
//! [ingest_limits]
//! max_deferred_total = 5
//! ```
//!
//! оставляет остальные лимиты кодовыми. Обратная совместимость: при
//! отсутствии файла [`SecurityConfig::load`] возвращает кодовые defaults.

use std::path::Path;

use serde::{Deserialize, Serialize};

use liveletters_mime::MimeLimits;
use liveletters_sync::{IngestLimits, RetentionPolicy};

use crate::ConfigError;

const SECURITY_CONFIG_FILENAME: &str = "config.toml";

/// Per-user политика безопасности: объединение всех квот и лимитов, которые
/// ранее жили как кодовые константы (`MimeLimits`, `IngestLimits`,
/// `RetentionPolicy`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub mime_limits: MimeLimits,
    #[serde(default)]
    pub ingest_limits: IngestLimits,
    #[serde(default)]
    pub retention: RetentionPolicy,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            schema_version: default_schema_version(),
            mime_limits: MimeLimits::default(),
            ingest_limits: IngestLimits::default(),
            retention: RetentionPolicy::default(),
        }
    }
}

fn default_schema_version() -> u32 {
    1
}

impl SecurityConfig {
    /// Читает `users/<name>/config.toml` из каталога `state_home` (per-user
    /// каталог, содержащий `lltt.db`). При отсутствии файла возвращает кодовые
    /// defaults, чтобы старые per-user каталоги и tempdir-тесты продолжали
    /// работать без миграции.
    pub fn load(state_home: &Path) -> Result<Self, ConfigError> {
        let path = state_home.join(SECURITY_CONFIG_FILENAME);
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(&path)?;
        let config: Self = toml::from_str(&raw)?;
        Ok(config)
    }

    /// Каноническое TOML-представление defaults. Используется при
    /// первоначальной записи файла в `lltt user add`. Не вызывает
    /// [`Self::load`], чтобы избежать цикла.
    pub fn default_toml() -> String {
        toml::to_string_pretty(&Self::default()).expect("SecurityConfig сериализуем в TOML")
    }

    /// Записывает defaults в `state_home/config.toml`, только если файла ещё
    /// нет. Идемпотентно: существующий файл не перезаписывается, чтобы
    /// уважать пользовательские правки (поведение «переопределение»).
    pub fn ensure_default_file(state_home: &Path) -> Result<(), ConfigError> {
        let path = state_home.join(SECURITY_CONFIG_FILENAME);
        if path.exists() {
            return Ok(());
        }
        std::fs::write(&path, Self::default_toml())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn load_missing_file_returns_defaults() {
        let tmp = TempDir::new().unwrap();
        let cfg = SecurityConfig::load(tmp.path()).unwrap();
        assert_eq!(cfg, SecurityConfig::default());
        assert_eq!(cfg.schema_version, 1);
        assert_eq!(cfg.ingest_limits.max_deferred_total, 100);
    }

    #[test]
    fn default_toml_round_trips() {
        let toml_str = SecurityConfig::default_toml();
        let parsed: SecurityConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed, SecurityConfig::default());
    }

    #[test]
    fn partial_override_uses_code_defaults_for_missing_keys() {
        let raw = r#"
schema_version = 1

[ingest_limits]
max_deferred_total = 5
"#;
        let parsed: SecurityConfig = toml::from_str(raw).unwrap();
        assert_eq!(parsed.ingest_limits.max_deferred_total, 5);
        // Остальные лимиты — из кодовых defaults.
        assert_eq!(
            parsed.ingest_limits.max_deferred_per_origin,
            IngestLimits::default().max_deferred_per_origin
        );
        assert_eq!(parsed.mime_limits, MimeLimits::default());
        assert_eq!(parsed.retention, RetentionPolicy::default());
    }

    #[test]
    fn ensure_default_file_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        SecurityConfig::ensure_default_file(tmp.path()).unwrap();
        let first = std::fs::read_to_string(tmp.path().join(SECURITY_CONFIG_FILENAME)).unwrap();
        // Повторный вызов не должен перезаписывать.
        std::fs::write(
            tmp.path().join(SECURITY_CONFIG_FILENAME),
            first.replace("max_parts = 8", "max_parts = 99"),
        )
        .unwrap();
        SecurityConfig::ensure_default_file(tmp.path()).unwrap();
        let second = std::fs::read_to_string(tmp.path().join(SECURITY_CONFIG_FILENAME)).unwrap();
        assert!(second.contains("max_parts = 99"));
        assert!(!second.contains("max_parts = 8"));
    }
}

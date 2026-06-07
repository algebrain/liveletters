//! Журнал сетевых операций и парсинга payload.
//!
//! API построено на обычных функциях (`log_info`, `log_error`, …) и
//! вспомогательном макросе [`format_args!`] для тех, кому важно не
//! аллоцировать `String` при выключенном журнале.
//!
//! По умолчанию выключен (`LogLevel::Off`); включается через
//! `${LIVELETTERS_HOME}/config.toml` (поле `log.level`) или командой
//! `lltt settings set log.level info`.
//!
//! Поведение:
//! - при `level = off` функции возвращают сразу (без форматирования);
//! - пишет в `${LIVELETTERS_HOME}/logs/liveletters.log` с ротацией по размеру
//!   (`max_size_bytes` / `keep_files`, дефолт 5 МиБ × 3);
//! - тела писем и payload никогда не пишутся, пока пользователь явно
//!   не поставит `log.include_bodies = true`; даже тогда — только через
//!   функцию `log_payload`.

pub mod config;
pub mod init;
pub mod level;
pub mod rotation;
pub mod writer;

pub use config::{LogConfig, LogDestination, LogLevel};
pub use init::{
    LogError, init, is_bodies_enabled, keep_files, max_size, reset_for_tests, shutdown,
};

/// Записать сообщение уровня `Error`.
pub fn log_error(message: impl AsRef<str>) {
    log(config::LogLevel::Error, message.as_ref());
}

/// Записать сообщение уровня `Warn`.
pub fn log_warn(message: impl AsRef<str>) {
    log(config::LogLevel::Warn, message.as_ref());
}

/// Записать сообщение уровня `Info`.
pub fn log_info(message: impl AsRef<str>) {
    log(config::LogLevel::Info, message.as_ref());
}

/// Записать сообщение уровня `Debug`.
pub fn log_debug(message: impl AsRef<str>) {
    log(config::LogLevel::Debug, message.as_ref());
}

/// Записать сообщение уровня `Trace`.
pub fn log_trace(message: impl AsRef<str>) {
    log(config::LogLevel::Trace, message.as_ref());
}

/// Записать тело письма / payload. Молча игнорируется, если
/// `log.include_bodies = false`.
pub fn log_payload(message: impl AsRef<str>) {
    if !is_bodies_enabled() {
        return;
    }
    log(config::LogLevel::Debug, message.as_ref());
}

fn log(level: config::LogLevel, message: &str) {
    if !level::is_enabled(level) {
        return;
    }
    writer::write_message(level, "", message, init::max_size(), init::keep_files());
}

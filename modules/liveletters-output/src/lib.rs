//! Общие функции вывода и ввода для команд `lltt`.
//!
//! Крейт живёт отдельно от [`apps/lltt`], чтобы команды могли переиспользовать
//! маскирование паролей, форматирование таблиц, парсинг `--visibility`
//! и чтение тела команды из файла или stdin без зависимости от бинаря.
//!
//! [`apps/lltt`]: ../../apps/lltt

mod context;
mod format;
mod io;
pub mod time;

pub use context::CommandContext;
pub use format::{mask_password, print_identity, print_kv, print_table};
pub use io::{body_was_empty, parse_visibility, read_body, read_body_io};
pub use time::format_unix_iso8601_utc;

/// Имя креЙта для диагностических сообщений и тестов.
pub fn crate_name() -> &'static str {
    "liveletters-output"
}

#[cfg(test)]
mod tests {
    use super::crate_name;

    #[test]
    fn exposes_crate_name() {
        assert_eq!(crate_name(), "liveletters-output");
    }
}

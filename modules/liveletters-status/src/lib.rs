//! Команда `lltt status` — краткий отчёт о состоянии домашнего каталога.

mod args;
mod error;
pub mod print;
mod run;

pub use args::Args;
pub use error::StatusError;
pub use liveletters_output::CommandContext;
pub use print::{StatusCounts, print_status};
pub use run::run;

/// Имя команды для clap-дерева и для диагностических сообщений.
pub const COMMAND_NAME: &str = "status";

/// Короткое описание команды, попадает в `lltt --help`.
pub fn summary() -> &'static str {
    "краткий отчёт о состоянии домашнего каталога"
}

/// Имя креЙта для логов и assert'ов.
pub fn crate_name() -> &'static str {
    "liveletters-status"
}

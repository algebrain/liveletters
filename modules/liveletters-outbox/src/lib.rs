//! Команда `lltt outbox` — список неотправленных событий (read-only).

mod args;
mod error;
mod run;

pub use args::{Args, OutboxAction};
pub use error::OutboxError;
pub use liveletters_output::CommandContext;
pub use run::{print_summary, run};

/// Имя команды для clap-дерева и для диагностических сообщений.
pub const COMMAND_NAME: &str = "outbox";

/// Короткое описание команды, попадает в `lltt --help`.
pub fn summary() -> &'static str {
    "показать неотправленные события"
}

/// Имя крейта для логов и assert'ов.
pub fn crate_name() -> &'static str {
    "liveletters-outbox"
}

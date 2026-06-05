//! Команда `lltt thread` — показать обсуждение (пост + комментарии) в виде дерева.

mod args;
mod error;
mod run;

pub use args::Args;
pub use error::ThreadError;
pub use liveletters_output::CommandContext;
pub use run::{print_thread, run};

/// Имя команды для clap-дерева и для диагностических сообщений.
pub const COMMAND_NAME: &str = "thread";

/// Короткое описание команды, попадает в `lltt --help`.
pub fn summary() -> &'static str {
    "показать обсуждение (пост + комментарии)"
}

/// Имя креЙта для логов и assert'ов.
pub fn crate_name() -> &'static str {
    "liveletters-thread"
}

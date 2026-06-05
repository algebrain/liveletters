//! Команда `lltt init` — подготовка домашнего каталога.

mod args;
mod error;
mod run;

pub use args::Args;
pub use error::InitError;
pub use liveletters_output::CommandContext;
pub use run::run;

/// Имя команды для clap-дерева и для диагностических сообщений.
pub const COMMAND_NAME: &str = "init";

/// Короткое описание команды, попадает в `lltt --help`.
pub fn summary() -> &'static str {
    "инициализировать домашний каталог"
}

/// Имя креЙта для логов и assert'ов.
pub fn crate_name() -> &'static str {
    "liveletters-init"
}

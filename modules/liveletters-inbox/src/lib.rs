//! Команда `lltt inbox` — управление входящей почтой.

mod args;
mod error;
pub mod import;
pub mod list;
mod run;
pub mod show;

pub use args::{Args, ImportArgs, InboxAction, ListArgs, ShowArgs};
pub use error::InboxError;
pub use liveletters_output::CommandContext;
pub use run::run;

/// Имя команды для clap-дерева и для диагностических сообщений.
pub const COMMAND_NAME: &str = "inbox";

/// Короткое описание команды, попадает в `lltt --help`.
pub fn summary() -> &'static str {
    "управление входящей почтой"
}

/// Имя крейта для логов и assert'ов.
pub fn crate_name() -> &'static str {
    "liveletters-inbox"
}

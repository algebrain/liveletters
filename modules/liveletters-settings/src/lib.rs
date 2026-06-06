//! Команда `lltt settings` — показать или изменить настройки.

mod args;
mod error;
pub mod print;
mod run;
pub mod set;
pub mod show;

pub use args::{Args, SettingsAction};
pub use error::SettingsError;
pub use liveletters_output::CommandContext;
pub use print::print_settings;
pub use run::run;

/// Имя команды для clap-дерева и для диагностических сообщений.
pub const COMMAND_NAME: &str = "settings";

/// Короткое описание команды, попадает в `lltt --help`.
pub fn summary() -> &'static str {
    "показать или изменить настройки"
}

/// Имя крейта для логов и assert'ов.
pub fn crate_name() -> &'static str {
    "liveletters-settings"
}

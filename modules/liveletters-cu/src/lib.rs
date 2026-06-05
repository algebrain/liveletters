//! Команда `lltt cu` — управление текущей и прочими идентичностями.

mod add;
mod args;
mod current;
mod error;
mod list;
mod name;
mod password_obfuscation;
mod rm;
mod run;
mod show;
mod switch;
mod user_init;

pub use args::{Args, CuAction};
pub use error::CuError;
pub use liveletters_output::CommandContext;
pub use run::{run, run_current, run_user};

/// Имя команды для clap-дерева и для диагностических сообщений.
pub const COMMAND_NAME: &str = "cu";

/// Короткое описание команды, попадает в `lltt --help`.
pub fn summary() -> &'static str {
    "управление текущим пользователем и списком идентичностей"
}

/// Имя креЙта для логов и assert'ов.
pub fn crate_name() -> &'static str {
    "liveletters-cu"
}

//! Команда `lltt feed` — показ ленты текущего пользователя liveletters.

mod args;
mod error;
mod print;
mod run;

pub use args::Args;
pub use error::FeedError;
pub use liveletters_output::CommandContext;
pub use print::print_feed;
pub use run::run;

/// Имя команды для clap-дерева и для диагностических сообщений.
pub const COMMAND_NAME: &str = "feed";

/// Короткое описание команды, попадает в `lltt --help`.
pub fn summary() -> &'static str {
    "показать ленту текущего пользователя liveletters"
}

/// Имя креЙта для логов и assert'ов.
pub fn crate_name() -> &'static str {
    "liveletters-feed"
}

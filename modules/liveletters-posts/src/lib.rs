//! Команда `lltt cu posts` — показ постов текущего пользователя liveletters.

mod args;
mod error;
mod print;
mod run;

pub use args::Args;
pub use error::PostsError;
pub use liveletters_output::CommandContext;
pub use print::print_posts;
pub use run::run;

/// Имя команды для clap-дерева и для диагностических сообщений.
pub const COMMAND_NAME: &str = "posts";

/// Короткое описание команды, попадает в `lltt --help`.
pub fn summary() -> &'static str {
    "показать посты текущего пользователя liveletters"
}

/// Имя крейта для логов и assert'ов.
pub fn crate_name() -> &'static str {
    "liveletters-posts"
}

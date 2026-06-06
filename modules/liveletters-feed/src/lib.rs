//! Команда `lltt feed` — лента постов из подписок текущего пользователя.

mod args;
mod error;
mod print;
mod run;

pub use args::Args;
pub use error::FeedError;
pub use liveletters_output::CommandContext;
pub use print::print_feed;
pub use run::run;

pub const COMMAND_NAME: &str = "feed";

pub fn summary() -> &'static str {
    "показать ленту подписок"
}

pub fn crate_name() -> &'static str {
    "liveletters-feed"
}

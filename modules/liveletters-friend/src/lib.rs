//! Команда `lltt friend` — добавить пользователя в друзья.

mod args;
mod error;
mod run;

pub use args::Args;
pub use error::FriendError;
pub use liveletters_output::CommandContext;
pub use run::run;

pub const COMMAND_NAME: &str = "friend";

pub fn summary() -> &'static str {
    "добавить пользователя в друзья"
}

pub fn crate_name() -> &'static str {
    "liveletters-friend"
}

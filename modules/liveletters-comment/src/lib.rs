//! Команда `lltt comment` — добавление комментария к записи.

mod args;
mod error;
mod run;

pub use args::{Args, CommentAction, NewArgs};
pub use error::CommentError;
pub use liveletters_output::CommandContext;
pub use run::run;

/// Имя команды для clap-дерева и для диагностических сообщений.
pub const COMMAND_NAME: &str = "comment";

/// Короткое описание команды, попадает в `lltt --help`.
pub fn summary() -> &'static str {
    "добавить комментарий к записи"
}

/// Имя крейта для логов и assert'ов.
pub fn crate_name() -> &'static str {
    "liveletters-comment"
}

/// Печатает сообщение о созданном комментарии.
pub fn print_created(comment_id: &str) {
    println!("комментарий создан: {comment_id}");
}

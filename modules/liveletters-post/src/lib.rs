//! Команда `lltt post` — создание новой записи в блоге текущего пользователя.

mod args;
mod error;
mod run;

pub use args::{Args, NewArgs, PostAction};
pub use error::PostError;
pub use liveletters_output::CommandContext;
pub use run::run;

/// Имя команды для clap-дерева и для диагностических сообщений.
pub const COMMAND_NAME: &str = "post";

/// Короткое описание команды, попадает в `lltt --help`.
pub fn summary() -> &'static str {
    "создать новую запись в блоге"
}

/// Имя крейта для логов и assert'ов.
pub fn crate_name() -> &'static str {
    "liveletters-post"
}

/// Печатает сообщение о созданной записи.
pub fn print_created(post_id: &str) {
    println!("запись создана: {post_id}");
}

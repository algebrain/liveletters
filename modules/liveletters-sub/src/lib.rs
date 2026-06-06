//! Команда `lltt sub` — управление подписками.

mod args;
mod error;
mod run;

pub use args::{Args, SubAction};
pub use error::SubError;
pub use liveletters_output::CommandContext;

/// Имя команды для clap-дерева и для диагностических сообщений.
pub const COMMAND_NAME: &str = "sub";

/// Короткое описание команды, попадает в `lltt --help`.
pub fn summary() -> &'static str {
    "управление подписками на блоги"
}

/// Имя крейта для логов и assert'ов.
pub fn crate_name() -> &'static str {
    "liveletters-sub"
}

/// Запуск команды `lltt sub`.
///
/// Принимает [`CommandContext`] и распарсенные [`Args`] (а не полную строку),
/// чтобы бинарь `apps/lltt` отвечал за clap-разбор, а крейт — за бизнес-логику.
pub fn run(
    ctx: &CommandContext,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let result = crate::run::run(ctx, args);
    result.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
}

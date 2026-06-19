//! Команда `lltt answer` — синоним `lltt comment new` (без флагов
//! `--post` / `--parent`, дискриминация по префиксу id).

#![allow(dead_code)] // COMMAND_NAME/summary/crate_name — для будущих расширений clap.

use std::error::Error;
use std::io::{self};

use liveletters_output::{CommandContext, read_body};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Id поста (начинается с «post-») или родительского комментария
    /// (начинается с «comment-»).
    pub target: String,

    /// Файл с телом комментария. Если не указан — тело читается из stdin.
    #[arg(long)]
    pub body_file: Option<std::path::PathBuf>,
}

pub fn run(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    let body = read_body(args.body_file.as_deref(), &mut io::stdin().lock())
        .map_err(Box::<dyn Error + Send + Sync>::from)?;
    let result = liveletters_comment::create(ctx, &args.target, &body)
        .map_err(Box::<dyn Error + Send + Sync>::from)?;
    liveletters_comment::print_created(result.comment().id().as_str());
    Ok(())
}

/// Имя команды для clap-дерева и для диагностических сообщений.
pub const COMMAND_NAME: &str = "answer";

/// Короткое описание команды, попадает в `lltt --help`.
pub fn summary() -> &'static str {
    "ответить на пост или на комментарий (id начинается с post- или comment-)"
}

/// Имя крейта для логов и assert'ов.
pub fn crate_name() -> &'static str {
    "liveletters-answer"
}

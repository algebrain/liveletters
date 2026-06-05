use std::path::PathBuf;

#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub action: PostAction,
}

#[derive(Debug, clap::Subcommand)]
pub enum PostAction {
    /// Создать новую запись.
    New(NewArgs),
}

#[derive(Debug, clap::Args)]
pub struct NewArgs {
    /// Файл с телом записи. Если не указан — тело читается из stdin.
    #[arg(long)]
    pub body_file: Option<PathBuf>,

    /// Уровень видимости: `public` или `friends_only`.
    #[arg(long, default_value = "public")]
    pub visibility: String,
}

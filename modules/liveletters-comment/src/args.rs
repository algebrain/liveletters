#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub action: CommentAction,
}

#[derive(Debug, clap::Subcommand)]
pub enum CommentAction {
    /// Создать новый комментарий.
    New(NewArgs),
}

#[derive(Debug, clap::Args)]
pub struct NewArgs {
    /// Идентификатор записи, к которой добавляется комментарий.
    #[arg(long)]
    pub post: String,

    /// Идентификатор родительского комментария (для вложенных ответов).
    #[arg(long)]
    pub parent: Option<String>,

    /// Файл с телом комментария. Если не указан — тело читается из stdin.
    #[arg(long)]
    pub body_file: Option<std::path::PathBuf>,

    /// Уровень видимости: `public` или `friends_only`.
    #[arg(long, default_value = "public")]
    pub visibility: String,
}

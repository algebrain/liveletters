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
    /// Id поста (начинается с «post-») или родительского комментария
    /// (начинается с «comment-»).
    pub target: String,

    /// Файл с телом комментария. Если не указан — тело читается из stdin.
    #[arg(long)]
    pub body_file: Option<std::path::PathBuf>,

    /// Уровень видимости: `public` или `friends_only`.
    #[arg(long, default_value = "public")]
    pub visibility: String,
}

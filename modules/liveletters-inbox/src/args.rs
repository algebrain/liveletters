use std::path::PathBuf;

#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub action: InboxAction,
}

#[derive(Debug, clap::Subcommand)]
pub enum InboxAction {
    /// Импортировать одно или несколько писем из .eml-файлов.
    Import(ImportArgs),
    /// Показать последние N входящих писем (по умолчанию 20).
    List(ListArgs),
    /// Показать полное тело одного письма по `message_id`.
    Show(ShowArgs),
}

#[derive(Debug, clap::Args)]
pub struct ImportArgs {
    #[arg(required = true)]
    pub files: Vec<PathBuf>,
}

#[derive(Debug, clap::Args)]
pub struct ListArgs {
    /// Фильтр по статусу: applied, duplicate, replay, unauthorized, invalid, malformed.
    #[arg(long)]
    pub status: Option<String>,
    /// Сколько последних писем показать (по умолчанию 20).
    #[arg(long, default_value_t = 20)]
    pub limit: usize,
}

#[derive(Debug, clap::Args)]
pub struct ShowArgs {
    /// Идентификатор сообщения (значение `Message-ID` или `message_id` в БД).
    pub id: String,
}

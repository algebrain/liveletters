use clap::Args as ClapArgs;

#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub action: Option<SyncAction>,
}

#[derive(Debug, clap::Subcommand)]
pub enum SyncAction {
    /// Забрать новые письма с IMAP и прогнать через SyncEngine.
    Pull,
    /// Отправить исходящие из outbox через SMTP.
    Push,
}

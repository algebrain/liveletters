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
    /// Разовый заброс: подтянуть письма за последние N суток,
    /// не сдвигая основной sync-курсор.
    Backfill {
        /// Сколько суток заглядывать назад. По умолчанию 30.
        #[arg(long, default_value_t = 30)]
        days: u32,
    },
}

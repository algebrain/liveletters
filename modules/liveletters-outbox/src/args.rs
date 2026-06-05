#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub action: OutboxAction,
}

#[derive(Debug, clap::Subcommand)]
pub enum OutboxAction {
    /// Показать неотправленные события (read-only).
    List,
}

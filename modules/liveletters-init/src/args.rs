#[derive(Debug, clap::Args)]
pub struct Args {
    /// Перезаписать существующий каталог, если он не пуст.
    #[arg(long)]
    pub force: bool,
}

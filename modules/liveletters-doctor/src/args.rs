#[derive(Debug, Default, clap::Args)]
pub struct Args {
    /// Расширенный вывод: deferred-события, identities, размер таблиц.
    #[arg(long, short = 'v')]
    pub verbose: bool,
}

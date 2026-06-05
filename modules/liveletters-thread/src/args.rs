#[derive(Debug, clap::Args)]
pub struct Args {
    /// Идентификатор записи, для которой нужно показать обсуждение.
    pub post_id: String,
}

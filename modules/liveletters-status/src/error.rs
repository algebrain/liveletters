#[derive(Debug, thiserror::Error)]
pub enum StatusError {
    #[error("ошибка хранилища: {0}")]
    Store(#[from] liveletters_store::StoreError),
}

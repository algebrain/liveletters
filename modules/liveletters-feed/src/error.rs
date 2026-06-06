#[derive(Debug, thiserror::Error)]
pub enum FeedError {
    #[error("ошибка конфигурации: {0}")]
    Config(#[from] liveletters_config::ConfigError),

    #[error("ошибка хранилища: {0}")]
    Store(#[from] liveletters_store::StoreError),
}

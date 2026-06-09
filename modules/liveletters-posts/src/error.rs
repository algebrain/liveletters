#[derive(Debug, thiserror::Error)]
pub enum PostsError {
    #[error("ошибка хранилища: {0}")]
    Store(#[from] liveletters_store::StoreError),

    #[error("ошибка прикладного слоя: {0}")]
    AppCore(#[from] liveletters_app_core::AppCoreError),

    #[error("ошибка конфигурации: {0}")]
    Config(#[from] liveletters_config::ConfigError),

    #[error("идентичность `{0}` не найдена в базе")]
    IdentityNotFound(String),
}

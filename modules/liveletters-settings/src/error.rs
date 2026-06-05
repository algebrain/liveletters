use thiserror::Error;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("ошибка хранилища: {0}")]
    Store(#[from] liveletters_store::StoreError),

    #[error("ошибка конфигурации: {0}")]
    Config(#[from] liveletters_config::ConfigError),

    #[error("неизвестный ключ: {0}")]
    InvalidKey(String),

    #[error("неверные аргументы: {0}")]
    InvalidArgs(String),
}

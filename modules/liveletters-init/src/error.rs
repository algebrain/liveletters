use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error("каталог {0} уже существует и не пуст; используйте --force")]
    AlreadyExists(PathBuf),

    #[error("ошибка хранилища: {0}")]
    Store(#[from] liveletters_store::StoreError),

    #[error("ошибка конфигурации: {0}")]
    Config(#[from] liveletters_config::ConfigError),

    #[error("ошибка секретного ключа: {0}")]
    Secret(#[from] liveletters_secret_box::SecretBoxError),

    #[error("ошибка ввода-вывода: {0}")]
    Io(#[from] std::io::Error),
}

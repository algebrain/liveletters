use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum PostError {
    #[error("ошибка хранилища: {0}")]
    Store(#[from] liveletters_store::StoreError),

    #[error("ошибка прикладного слоя: {0}")]
    AppCore(#[from] liveletters_app_core::AppCoreError),

    #[error("ошибка конфигурации: {0}")]
    Config(#[from] liveletters_config::ConfigError),

    #[error("ошибка ввода-вывода: {0}")]
    Io(#[from] io::Error),

    #[error("{0}")]
    IoFromOutput(String),

    #[error("файл с телом записи не найден: {path}")]
    BodyFileNotFound { path: PathBuf },

    #[error("{0}")]
    UnknownVisibility(String),

    #[error("тело записи пустое")]
    EmptyBody,
}

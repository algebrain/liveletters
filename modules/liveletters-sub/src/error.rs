use liveletters_app_core::AppCoreError;
use liveletters_config::ConfigError;
use liveletters_domain::DomainError;
use liveletters_store::StoreError;

#[derive(Debug, thiserror::Error)]
pub enum SubError {
    #[error("ошибка конфигурации: {0}")]
    Config(#[from] ConfigError),

    #[error("ошибка ввода-вывода: {0}")]
    Io(#[from] std::io::Error),

    #[error("ошибка доменной валидации: {0}")]
    Domain(#[from] DomainError),

    #[error("ошибка хранилища: {0}")]
    Store(#[from] StoreError),

    #[error("ошибка приложения: {0}")]
    AppCore(#[from] AppCoreError),

    #[error("неизвестная подкоманда или неверные аргументы: {0}")]
    InvalidArgs(String),
}

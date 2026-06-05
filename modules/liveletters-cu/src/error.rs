use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum CuError {
    #[error("ошибка конфигурации: {0}")]
    Config(#[from] liveletters_config::ConfigError),

    #[error("ошибка ввода-вывода: {0}")]
    Io(#[from] std::io::Error),

    #[error("ошибка хранилища: {0}")]
    Store(#[from] liveletters_store::StoreError),

    #[error("ошибка секретного ключа: {0}")]
    Secret(#[from] liveletters_secret_box::SecretBoxError),

    #[error("ошибка ввода пароля: {0}")]
    Prompt(String),

    #[error("источник {0} не найден")]
    FromFileMissing(PathBuf),

    #[error("нельзя удалить текущую идентичность `{0}`; сначала переключитесь")]
    CannotRemoveCurrent(String),

    #[error("неизвестная подкоманда или неверные аргументы: {0}")]
    InvalidArgs(String),

    #[error("эта операция перенесена в `{0}`")]
    UseUserCommand(String),

    #[error("пароль для {0} не совпадает с подтверждением")]
    PasswordConfirmationMismatch(&'static str),

    #[error("нельзя совмещать подкоманду и имя переключения")]
    ConflictingArgs,
}

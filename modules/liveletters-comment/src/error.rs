use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum CommentError {
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

    #[error("файл с телом комментария не найден: {path}")]
    BodyFileNotFound { path: PathBuf },

    #[error("{0}")]
    UnknownVisibility(String),

    #[error("тело комментария пустое")]
    EmptyBody,

    #[error("идентичность `{0}` не найдена в базе")]
    IdentityNotFound(String),

    #[error("пост «{0}» не найден")]
    PostNotFound(String),

    #[error("комментарий «{0}» не найден")]
    CommentNotFound(String),

    #[error("id «{0}» должен начинаться с «post-» или «comment-»")]
    InvalidTarget(String),
}

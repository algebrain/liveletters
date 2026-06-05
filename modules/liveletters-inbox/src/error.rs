use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum InboxError {
    #[error("ошибка хранилища: {0}")]
    Store(#[from] liveletters_store::StoreError),

    #[error("ошибка MIME-разбора: {0}")]
    Mime(#[from] liveletters_mime::MimeError),

    #[error("ошибка синхронизации: {0}")]
    Sync(#[from] liveletters_sync::SyncError),

    #[error("ошибка ввода-вывода: {0}")]
    Io(#[from] std::io::Error),

    #[error("файл {0} не найден")]
    FileNotFound(PathBuf),

    #[error(
        "неизвестный статус: {0}; допустимые: applied, duplicate, replay, unauthorized, invalid, malformed"
    )]
    InvalidStatus(String),

    #[error("сообщение с id {0} не найдено в raw_messages")]
    MessageNotFound(String),
}

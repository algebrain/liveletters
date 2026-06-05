use liveletters_domain::DomainError;
use liveletters_protocol::ProtocolError;
use liveletters_store::StoreError;
use liveletters_sync::SyncError;

#[derive(Debug)]
pub enum AppCoreError {
    Domain(DomainError),
    Protocol(ProtocolError),
    Sync(SyncError),
    Store(StoreError),
    SettingsValidation { field: String, message: String },
    PostNotFound { post_id: String },
    CommentNotFound { comment_id: String },
}

impl std::fmt::Display for AppCoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Domain(error) => write!(f, "ошибка доменной модели: {error:?}"),
            Self::Protocol(error) => write!(f, "ошибка протокола: {error:?}"),
            Self::Sync(error) => write!(f, "ошибка синхронизации: {error:?}"),
            Self::Store(error) => write!(f, "ошибка хранилища: {error}"),
            Self::SettingsValidation { field, message } => {
                write!(f, "ошибка настроек `{field}`: {message}")
            }
            Self::PostNotFound { post_id } => write!(f, "пост `{post_id}` не найден"),
            Self::CommentNotFound { comment_id } => {
                write!(f, "комментарий `{comment_id}` не найден")
            }
        }
    }
}

impl std::error::Error for AppCoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<StoreError> for AppCoreError {
    fn from(value: StoreError) -> Self {
        Self::Store(value)
    }
}

impl From<ProtocolError> for AppCoreError {
    fn from(value: ProtocolError) -> Self {
        Self::Protocol(value)
    }
}

impl From<SyncError> for AppCoreError {
    fn from(value: SyncError) -> Self {
        Self::Sync(value)
    }
}

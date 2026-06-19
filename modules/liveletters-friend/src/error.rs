#[derive(Debug, thiserror::Error)]
pub enum FriendError {
    #[error("ошибка хранилища: {0}")]
    Store(#[from] liveletters_store::StoreError),

    #[error("ошибка приложения: {0}")]
    AppCore(#[from] liveletters_app_core::AppCoreError),

    #[error("идентичность `{0}` не найдена в базе")]
    IdentityNotFound(String),
}

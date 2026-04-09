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
    PostNotFound { post_id: String },
    CommentNotFound { comment_id: String },
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

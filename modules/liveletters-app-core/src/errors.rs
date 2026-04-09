use liveletters_domain::DomainError;
use liveletters_protocol::ProtocolError;
use liveletters_store::StoreError;

#[derive(Debug)]
pub enum AppCoreError {
    Domain(DomainError),
    Protocol(ProtocolError),
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

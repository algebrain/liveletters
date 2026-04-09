#[derive(Debug)]
pub enum SyncError {
    Store(liveletters_store::StoreError),
    SerializePayload(serde_json::Error),
}

impl From<liveletters_store::StoreError> for SyncError {
    fn from(value: liveletters_store::StoreError) -> Self {
        Self::Store(value)
    }
}

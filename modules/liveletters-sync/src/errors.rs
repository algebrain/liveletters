#[derive(Debug)]
pub enum SyncError {
    Store(liveletters_store::StoreError),
    SerializePayload(serde_json::Error),
    DeserializePayload(serde_json::Error),
}

impl From<liveletters_store::StoreError> for SyncError {
    fn from(value: liveletters_store::StoreError) -> Self {
        Self::Store(value)
    }
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(inner) => write!(f, "store: {inner}"),
            Self::SerializePayload(inner) => write!(f, "serialize payload: {inner}"),
            Self::DeserializePayload(inner) => write!(f, "deserialize payload: {inner}"),
        }
    }
}

impl std::error::Error for SyncError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Store(inner) => Some(inner),
            Self::SerializePayload(inner) | Self::DeserializePayload(inner) => Some(inner),
        }
    }
}

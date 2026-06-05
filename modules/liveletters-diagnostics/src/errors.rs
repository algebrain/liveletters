#[derive(Debug)]
pub enum DiagnosticsError {
    Store(liveletters_store::StoreError),
}

impl From<liveletters_store::StoreError> for DiagnosticsError {
    fn from(value: liveletters_store::StoreError) -> Self {
        Self::Store(value)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DoctorError {
    #[error("ошибка хранилища: {0}")]
    Store(#[from] liveletters_store::StoreError),

    #[error("ошибка диагностики: {0}")]
    Diagnostics(String),
}

impl From<liveletters_diagnostics::DiagnosticsError> for DoctorError {
    fn from(value: liveletters_diagnostics::DiagnosticsError) -> Self {
        Self::Diagnostics(format!("{value:?}"))
    }
}

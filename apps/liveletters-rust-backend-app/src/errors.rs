#[derive(Debug)]
pub enum BackendError {
    AppCore(liveletters_app_core::AppCoreError),
    Diagnostics(liveletters_diagnostics::DiagnosticsError),
    Store(liveletters_store::StoreError),
}

impl From<liveletters_app_core::AppCoreError> for BackendError {
    fn from(value: liveletters_app_core::AppCoreError) -> Self {
        Self::AppCore(value)
    }
}

impl From<liveletters_diagnostics::DiagnosticsError> for BackendError {
    fn from(value: liveletters_diagnostics::DiagnosticsError) -> Self {
        Self::Diagnostics(value)
    }
}

impl From<liveletters_store::StoreError> for BackendError {
    fn from(value: liveletters_store::StoreError) -> Self {
        Self::Store(value)
    }
}

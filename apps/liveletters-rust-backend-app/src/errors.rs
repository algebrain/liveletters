use serde::Serialize;

#[derive(Debug)]
pub enum BackendError {
    AppCore(liveletters_app_core::AppCoreError),
    Diagnostics(liveletters_diagnostics::DiagnosticsError),
    Store(liveletters_store::StoreError),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommandErrorDto {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
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

impl From<BackendError> for CommandErrorDto {
    fn from(value: BackendError) -> Self {
        match value {
            BackendError::AppCore(liveletters_app_core::AppCoreError::PostNotFound { post_id }) => {
                Self {
                    code: "not_found".into(),
                    message: format!("post not found: {post_id}"),
                    details: Some(post_id),
                }
            }
            BackendError::AppCore(liveletters_app_core::AppCoreError::CommentNotFound {
                comment_id,
            }) => Self {
                code: "not_found".into(),
                message: format!("comment not found: {comment_id}"),
                details: Some(comment_id),
            },
            BackendError::AppCore(liveletters_app_core::AppCoreError::Domain(error)) => Self {
                code: "validation_error".into(),
                message: format!("{error:?}"),
                details: None,
            },
            BackendError::AppCore(error) => Self {
                code: "app_core_error".into(),
                message: format!("{error:?}"),
                details: None,
            },
            BackendError::Diagnostics(error) => Self {
                code: "diagnostics_error".into(),
                message: format!("{error:?}"),
                details: None,
            },
            BackendError::Store(error) => Self {
                code: "store_error".into(),
                message: format!("{error:?}"),
                details: None,
            },
        }
    }
}

#[cfg(feature = "tauri-runtime")]
impl CommandErrorDto {
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            code: "internal_error".into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn emit(error: tauri::Error) -> Self {
        Self {
            code: "emit_error".into(),
            message: format!("{error:?}"),
            details: None,
        }
    }
}

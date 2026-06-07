use thiserror::Error;

#[derive(Debug, Error)]
pub enum I18nError {
    #[error("unknown locale: {0}")]
    UnknownLocale(String),
    #[error("unknown i18n key: {0}")]
    UnknownKey(String),
    #[error("missing variable {name} for key {key}")]
    MissingVariable { key: String, name: String },
}

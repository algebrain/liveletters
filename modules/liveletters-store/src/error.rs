#[derive(Debug)]
pub enum StoreError {
    Sqlite(rusqlite::Error),
    Io(std::io::Error),
    MissingHomeDirectory,
    ProtectedSecretUnavailable { message: String },
    InvalidProtectedSecretFormat { message: String },
    InvalidColumn(String),
    InvalidTable(String),
    AuthorNotFound { email: String },
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "sqlite: {error}"),
            Self::Io(error) => write!(f, "io: {error}"),
            Self::MissingHomeDirectory => write!(f, "домашний каталог не указан"),
            Self::ProtectedSecretUnavailable { message } => {
                write!(f, "секрет недоступен: {message}")
            }
            Self::InvalidProtectedSecretFormat { message } => {
                write!(f, "неверный формат секрета: {message}")
            }
            Self::InvalidColumn(column) => write!(f, "неизвестная колонка: {column}"),
            Self::InvalidTable(table) => write!(f, "неизвестная таблица: {table}"),
            Self::AuthorNotFound { email } => write!(f, "автор не найден: {email}"),
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for StoreError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<std::io::Error> for StoreError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

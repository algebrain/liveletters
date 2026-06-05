#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    BlankIdentifier(&'static str),
    BlankBody(&'static str),
    InvalidAddress,
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BlankIdentifier(field) => write!(f, "пустой идентификатор `{field}`"),
            Self::BlankBody(field) => write!(f, "пустое тело `{field}`"),
            Self::InvalidAddress => write!(f, "некорректный почтовый адрес (нет `@`)"),
        }
    }
}

impl std::error::Error for DomainError {}

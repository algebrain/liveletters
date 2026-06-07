#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportError {
    AuthenticationFailed,
    Network(String),
    InvalidEmailFormat(&'static str),
    MissingHumanReadablePart,
    MissingTechnicalPart,
    Protocol(String),
    UnexpectedResponse(String),
    UnsupportedAuthMechanism(&'static str),
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AuthenticationFailed => f.write_str("ошибка аутентификации"),
            Self::Network(msg) => write!(f, "сетевая ошибка: {msg}"),
            Self::InvalidEmailFormat(s) => write!(f, "некорректный формат адреса: {s}"),
            Self::MissingHumanReadablePart => f.write_str("письмо без человекочитаемой части"),
            Self::MissingTechnicalPart => f.write_str("письмо без технической части"),
            Self::Protocol(msg) => write!(f, "ошибка протокола: {msg}"),
            Self::UnexpectedResponse(msg) => write!(f, "неожиданный ответ сервера: {msg}"),
            Self::UnsupportedAuthMechanism(s) => write!(f, "неподдерживаемая AUTH-механика: {s}"),
        }
    }
}

impl std::error::Error for TransportError {}

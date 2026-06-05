#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MimeError {
    Protocol(String),
    InvalidEmailFormat(&'static str),
    MissingHumanReadablePart,
    MissingTechnicalPart,
}

impl std::fmt::Display for MimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Protocol(message) => write!(f, "protocol: {message}"),
            Self::InvalidEmailFormat(message) => write!(f, "invalid email format: {message}"),
            Self::MissingHumanReadablePart => write!(f, "missing human readable part"),
            Self::MissingTechnicalPart => write!(f, "missing technical part"),
        }
    }
}

impl std::error::Error for MimeError {}

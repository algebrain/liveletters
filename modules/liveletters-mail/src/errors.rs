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

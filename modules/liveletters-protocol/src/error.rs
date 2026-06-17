#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    BlankEnvelopeField(&'static str),
    BlankHumanReadableBody,
    InvalidIdentity(String),
    MalformedJson(String),
}

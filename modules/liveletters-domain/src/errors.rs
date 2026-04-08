#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainError {
    BlankIdentifier(&'static str),
    BlankBody(&'static str),
}

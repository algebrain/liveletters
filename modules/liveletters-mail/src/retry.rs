use crate::TransportError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MailRetryPolicy {
    max_attempts: usize,
}

impl MailRetryPolicy {
    pub fn new(max_attempts: usize) -> Self {
        Self { max_attempts }
    }

    pub fn should_retry(&self, error: &TransportError) -> bool {
        matches!(
            error,
            TransportError::Network(_) | TransportError::UnexpectedResponse(_)
        )
    }

    pub fn allows_attempt(&self, attempt_number: usize) -> bool {
        attempt_number <= self.max_attempts
    }
}

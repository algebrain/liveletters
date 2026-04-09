use crate::{OutgoingEmail, TransportError};

#[derive(Debug, Default)]
pub struct InMemorySmtpTransport {
    sent_emails: Vec<OutgoingEmail>,
}

impl InMemorySmtpTransport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn send(&mut self, email: OutgoingEmail) -> Result<(), TransportError> {
        self.sent_emails.push(email);
        Ok(())
    }

    pub fn sent_emails(&self) -> &[OutgoingEmail] {
        &self.sent_emails
    }
}

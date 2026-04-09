use crate::{ReceivedEmail, TransportError};

#[derive(Debug, Default)]
pub struct InMemoryImapMailbox {
    queued_emails: Vec<ReceivedEmail>,
}

impl InMemoryImapMailbox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_raw_email(&mut self, message_id: &str, raw_message: &str) {
        self.queued_emails.push(ReceivedEmail {
            message_id: message_id.to_owned(),
            raw_message: raw_message.to_owned(),
        });
    }

    pub fn fetch_new(&mut self) -> Result<Vec<ReceivedEmail>, TransportError> {
        Ok(std::mem::take(&mut self.queued_emails))
    }
}

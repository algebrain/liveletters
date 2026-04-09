#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutgoingEmail {
    pub from: String,
    pub to: String,
    pub subject: String,
    pub raw_message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceivedEmail {
    pub message_id: String,
    pub raw_message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEmail {
    headers: Vec<(String, String)>,
    body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedMailParts {
    human_readable_body: String,
    technical_body: String,
}

impl ParsedEmail {
    pub fn new(headers: Vec<(String, String)>, body: String) -> Self {
        Self { headers, body }
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    pub fn subject(&self) -> Option<String> {
        self.header("Subject").map(ToOwned::to_owned)
    }
}

impl ExtractedMailParts {
    pub fn new(human_readable_body: String, technical_body: String) -> Self {
        Self {
            human_readable_body,
            technical_body,
        }
    }

    pub fn human_readable_body(&self) -> &str {
        &self.human_readable_body
    }

    pub fn technical_body(&self) -> &str {
        &self.technical_body
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MailAuth {
    None,
    Password {
        username: String,
        password: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmtpTransportConfig {
    server: String,
    port: u16,
    hello_domain: String,
    auth: MailAuth,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImapMailboxConfig {
    server: String,
    port: u16,
    mailbox: String,
    auth: MailAuth,
}

impl SmtpTransportConfig {
    pub fn new(server: impl Into<String>, port: u16, hello_domain: impl Into<String>, auth: MailAuth) -> Self {
        Self {
            server: server.into(),
            port,
            hello_domain: hello_domain.into(),
            auth,
        }
    }

    pub fn server(&self) -> &str {
        &self.server
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn hello_domain(&self) -> &str {
        &self.hello_domain
    }

    pub fn auth(&self) -> &MailAuth {
        &self.auth
    }
}

impl ImapMailboxConfig {
    pub fn new(server: impl Into<String>, port: u16, mailbox: impl Into<String>, auth: MailAuth) -> Self {
        Self {
            server: server.into(),
            port,
            mailbox: mailbox.into(),
            auth,
        }
    }

    pub fn server(&self) -> &str {
        &self.server
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn mailbox(&self) -> &str {
        &self.mailbox
    }

    pub fn auth(&self) -> &MailAuth {
        &self.auth
    }
}

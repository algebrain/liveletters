#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MailAuth {
    None,
    Password {
        username: String,
        password: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MailSecurity {
    None,
    StartTls,
    Tls,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmtpTransportConfig {
    server: String,
    port: u16,
    hello_domain: String,
    security: MailSecurity,
    auth: MailAuth,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImapMailboxConfig {
    server: String,
    port: u16,
    mailbox: String,
    security: MailSecurity,
    auth: MailAuth,
}

impl MailSecurity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::StartTls => "starttls",
            Self::Tls => "tls",
        }
    }
}

impl SmtpTransportConfig {
    pub fn new(
        server: impl Into<String>,
        port: u16,
        hello_domain: impl Into<String>,
        security: MailSecurity,
        auth: MailAuth,
    ) -> Self {
        Self {
            server: server.into(),
            port,
            hello_domain: hello_domain.into(),
            security,
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

    pub fn security(&self) -> MailSecurity {
        self.security
    }

    pub fn auth(&self) -> &MailAuth {
        &self.auth
    }
}

impl ImapMailboxConfig {
    pub fn new(
        server: impl Into<String>,
        port: u16,
        mailbox: impl Into<String>,
        security: MailSecurity,
        auth: MailAuth,
    ) -> Self {
        Self {
            server: server.into(),
            port,
            mailbox: mailbox.into(),
            security,
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

    pub fn security(&self) -> MailSecurity {
        self.security
    }

    pub fn auth(&self) -> &MailAuth {
        &self.auth
    }
}

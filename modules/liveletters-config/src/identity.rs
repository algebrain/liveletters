use liveletters_domain::ResourceAddress;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdentityConfig {
    pub account_id: String,
    pub display_name: String,
    pub mail: MailSettings,
    #[serde(default)]
    pub meta: IdentityMeta,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdentityMeta {
    #[serde(default)]
    pub resources_owned: Vec<String>,
    #[serde(default)]
    pub subscriptions: Vec<ResourceAddress>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MailSettings {
    pub publish: String,
    #[serde(default)]
    pub receive: Vec<String>,
    #[serde(default)]
    pub smtp: Option<SmtpSettings>,
    #[serde(default)]
    pub imap: Option<ImapSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SmtpSettings {
    pub host: String,
    pub port: u16,
    pub security: MailSecurity,
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_pwd_obfuscate")]
    pub pwd_obfuscate: bool,
    #[serde(default)]
    pub hello_domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImapSettings {
    pub host: String,
    pub port: u16,
    pub security: MailSecurity,
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_pwd_obfuscate")]
    pub pwd_obfuscate: bool,
    #[serde(default = "default_mailbox")]
    pub mailbox: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MailSecurity {
    None,
    StartTls,
    Tls,
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

fn default_mailbox() -> String {
    "INBOX".to_owned()
}

fn default_pwd_obfuscate() -> bool {
    true
}

impl IdentityConfig {
    pub fn account_id(&self) -> &str {
        &self.account_id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn mail(&self) -> &MailSettings {
        &self.mail
    }

    pub fn resources_owned(&self) -> &[String] {
        &self.meta.resources_owned
    }

    pub fn subscriptions(&self) -> &[ResourceAddress] {
        &self.meta.subscriptions
    }
}

impl MailSettings {
    pub fn publish(&self) -> &str {
        &self.publish
    }

    pub fn receive(&self) -> &[String] {
        &self.receive
    }

    pub fn smtp(&self) -> Option<&SmtpSettings> {
        self.smtp.as_ref()
    }

    pub fn imap(&self) -> Option<&ImapSettings> {
        self.imap.as_ref()
    }
}

impl SmtpSettings {
    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn security(&self) -> MailSecurity {
        self.security
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn pwd_obfuscate(&self) -> bool {
        self.pwd_obfuscate
    }

    pub fn hello_domain(&self) -> &str {
        &self.hello_domain
    }
}

impl ImapSettings {
    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn security(&self) -> MailSecurity {
        self.security
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn pwd_obfuscate(&self) -> bool {
        self.pwd_obfuscate
    }

    pub fn mailbox(&self) -> &str {
        &self.mailbox
    }
}

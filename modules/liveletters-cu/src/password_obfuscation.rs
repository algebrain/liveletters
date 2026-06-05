use liveletters_config::IdentityConfig;
use liveletters_secret_box::{SecretBox, default_key_path};

use crate::CuError;

pub trait PasswordConfirmer {
    fn confirm(&mut self, label: &'static str) -> Result<String, CuError>;
}

pub struct DialoguerPasswordConfirmer;

impl PasswordConfirmer for DialoguerPasswordConfirmer {
    fn confirm(&mut self, label: &'static str) -> Result<String, CuError> {
        read_masked_password(label)
    }
}

fn read_masked_password(label: &'static str) -> Result<String, CuError> {
    use console::Key;

    let term = console::Term::stderr();
    term.write_str(&format!("повторите пароль для {label}: "))
        .map_err(|error| CuError::Prompt(error.to_string()))?;

    let mut password = String::new();
    loop {
        match term
            .read_key()
            .map_err(|error| CuError::Prompt(error.to_string()))?
        {
            Key::Enter => {
                term.write_line("")
                    .map_err(|error| CuError::Prompt(error.to_string()))?;
                return Ok(password);
            }
            Key::Backspace => {
                if !password.is_empty() {
                    password.pop();
                    term.write_str("\u{8} \u{8}")
                        .map_err(|error| CuError::Prompt(error.to_string()))?;
                }
            }
            Key::Char(ch) => {
                password.push(ch);
                term.write_str("*")
                    .map_err(|error| CuError::Prompt(error.to_string()))?;
            }
            _ => {}
        }
    }
}

pub fn obfuscate_identity_passwords(
    home: &std::path::Path,
    cfg: &mut IdentityConfig,
    confirmer: &mut dyn PasswordConfirmer,
) -> Result<bool, CuError> {
    let secret_box = SecretBox::open_or_create(&default_key_path(home))?;
    let mut changed = false;

    if let Some(smtp) = cfg.mail.smtp.as_mut()
        && should_obfuscate(smtp.pwd_obfuscate, &smtp.password)
    {
        confirm_password("SMTP", &smtp.password, confirmer)?;
        smtp.password = secret_box.obfuscate(&smtp.password)?;
        changed = true;
    }

    if let Some(imap) = cfg.mail.imap.as_mut()
        && should_obfuscate(imap.pwd_obfuscate, &imap.password)
    {
        confirm_password("IMAP", &imap.password, confirmer)?;
        imap.password = secret_box.obfuscate(&imap.password)?;
        changed = true;
    }

    Ok(changed)
}

fn should_obfuscate(enabled: bool, password: &str) -> bool {
    enabled && !password.is_empty() && !SecretBox::is_obfuscated(password)
}

fn confirm_password(
    label: &'static str,
    password: &str,
    confirmer: &mut dyn PasswordConfirmer,
) -> Result<(), CuError> {
    let confirmation = confirmer.confirm(label)?;
    if confirmation != password {
        return Err(CuError::PasswordConfirmationMismatch(label));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use liveletters_config::{
        IdentityConfig, IdentityMeta, ImapSettings, MailSecurity, MailSettings, SmtpSettings,
    };
    use liveletters_secret_box::SecretBox;
    use tempfile::TempDir;

    use super::*;

    struct FixedConfirmer {
        values: std::collections::VecDeque<String>,
    }

    impl FixedConfirmer {
        fn new(values: &[&str]) -> Self {
            Self {
                values: values.iter().map(|value| value.to_string()).collect(),
            }
        }
    }

    impl PasswordConfirmer for FixedConfirmer {
        fn confirm(&mut self, _label: &'static str) -> Result<String, CuError> {
            Ok(self.values.pop_front().unwrap_or_default())
        }
    }

    #[test]
    fn obfuscates_smtp_and_imap_passwords_after_confirmation() {
        let tmp = TempDir::new().unwrap();
        let mut cfg = identity_with_passwords("smtp-secret", "imap-secret", true);
        let mut confirmer = FixedConfirmer::new(&["smtp-secret", "imap-secret"]);

        let changed = obfuscate_identity_passwords(tmp.path(), &mut cfg, &mut confirmer).unwrap();

        assert!(changed);
        let smtp = cfg.mail.smtp.as_ref().unwrap();
        let imap = cfg.mail.imap.as_ref().unwrap();
        assert!(SecretBox::is_obfuscated(&smtp.password));
        assert!(SecretBox::is_obfuscated(&imap.password));
        assert_ne!(smtp.password, "smtp-secret");
        assert_ne!(imap.password, "imap-secret");
    }

    #[test]
    fn confirmation_mismatch_errors_without_changing_passwords() {
        let tmp = TempDir::new().unwrap();
        let mut cfg = identity_with_passwords("smtp-secret", "imap-secret", true);
        let mut confirmer = FixedConfirmer::new(&["wrong"]);

        let err = obfuscate_identity_passwords(tmp.path(), &mut cfg, &mut confirmer).unwrap_err();

        assert!(matches!(err, CuError::PasswordConfirmationMismatch("SMTP")));
        assert_eq!(cfg.mail.smtp.as_ref().unwrap().password, "smtp-secret");
    }

    #[test]
    fn disabled_obfuscation_leaves_plain_passwords() {
        let tmp = TempDir::new().unwrap();
        let mut cfg = identity_with_passwords("smtp-secret", "imap-secret", false);
        let mut confirmer = FixedConfirmer::new(&[]);

        let changed = obfuscate_identity_passwords(tmp.path(), &mut cfg, &mut confirmer).unwrap();

        assert!(!changed);
        assert_eq!(cfg.mail.smtp.as_ref().unwrap().password, "smtp-secret");
        assert_eq!(cfg.mail.imap.as_ref().unwrap().password, "imap-secret");
    }

    fn identity_with_passwords(
        smtp_password: &str,
        imap_password: &str,
        pwd_obfuscate: bool,
    ) -> IdentityConfig {
        IdentityConfig {
            account_id: "acct_alice".to_owned(),
            display_name: "Alice".to_owned(),
            mail: MailSettings {
                publish: "alice@example.org".to_owned(),
                receive: vec!["alice@example.org".to_owned()],
                smtp: Some(SmtpSettings {
                    host: "smtp.example.org".to_owned(),
                    port: 587,
                    security: MailSecurity::StartTls,
                    username: "alice@example.org".to_owned(),
                    password: smtp_password.to_owned(),
                    pwd_obfuscate,
                    hello_domain: "example.org".to_owned(),
                }),
                imap: Some(ImapSettings {
                    host: "imap.example.org".to_owned(),
                    port: 993,
                    security: MailSecurity::Tls,
                    username: "alice@example.org".to_owned(),
                    password: imap_password.to_owned(),
                    pwd_obfuscate,
                    mailbox: "INBOX".to_owned(),
                }),
            },
            meta: IdentityMeta::default(),
        }
    }
}

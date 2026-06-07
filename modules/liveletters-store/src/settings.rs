use liveletters_secret_box::SecretBox;
use rusqlite::params;

use crate::{MailSettingsRecord, Store, StoreError, UserSettingsRecord, secret_bridge};

impl Store {
    pub fn save_user_settings_record(&self, record: &UserSettingsRecord) -> Result<(), StoreError> {
        self.connection().execute(
            r#"
            INSERT OR REPLACE INTO user_settings
                (profile_id, nickname, email_address, avatar_url, language, setup_completed)
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                record.profile_id,
                record.nickname,
                record.email_address,
                record.avatar_url,
                record.language,
                record.setup_completed as i64,
            ],
        )?;

        Ok(())
    }

    pub fn get_user_settings_record(
        &self,
        profile_id: &str,
    ) -> Result<Option<UserSettingsRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            r#"
            SELECT profile_id, nickname, email_address, avatar_url, language, setup_completed
            FROM user_settings
            WHERE profile_id = ?1
            "#,
        )?;

        let mut rows = stmt.query([profile_id])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };

        Ok(Some(UserSettingsRecord {
            profile_id: row.get(0)?,
            nickname: row.get(1)?,
            email_address: row.get(2)?,
            avatar_url: row.get(3)?,
            language: row.get(4)?,
            setup_completed: row.get::<_, i64>(5)? != 0,
        }))
    }

    pub fn save_mail_settings_record(&self, record: &MailSettingsRecord) -> Result<(), StoreError> {
        let smtp_password = self.obfuscate_secret_if_needed(&record.smtp_password)?;
        let imap_password = self.obfuscate_secret_if_needed(&record.imap_password)?;

        self.connection().execute(
            r#"
            INSERT OR REPLACE INTO mail_settings
                (
                    profile_id,
                    smtp_host,
                    smtp_port,
                    smtp_security,
                    smtp_username,
                    smtp_password,
                    smtp_hello_domain,
                    imap_host,
                    imap_port,
                    imap_security,
                    imap_username,
                    imap_password,
                    imap_mailbox
                )
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
            params![
                record.profile_id,
                record.smtp_host,
                record.smtp_port as i64,
                record.smtp_security,
                record.smtp_username,
                smtp_password,
                record.smtp_hello_domain,
                record.imap_host,
                record.imap_port as i64,
                record.imap_security,
                record.imap_username,
                imap_password,
                record.imap_mailbox,
            ],
        )?;

        Ok(())
    }

    pub fn get_mail_settings_record(
        &self,
        profile_id: &str,
    ) -> Result<Option<MailSettingsRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            r#"
            SELECT
                profile_id,
                smtp_host,
                smtp_port,
                smtp_security,
                smtp_username,
                smtp_password,
                smtp_hello_domain,
                imap_host,
                imap_port,
                imap_security,
                imap_username,
                imap_password,
                imap_mailbox
            FROM mail_settings
            WHERE profile_id = ?1
            "#,
        )?;

        let mut rows = stmt.query([profile_id])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };

        let smtp_password: String = row.get(5)?;
        let imap_password: String = row.get(11)?;

        let smtp_password =
            self.reveal_secret_with_lazy_migration(profile_id, "smtp_password", &smtp_password)?;
        let imap_password =
            self.reveal_secret_with_lazy_migration(profile_id, "imap_password", &imap_password)?;

        Ok(Some(MailSettingsRecord {
            profile_id: row.get(0)?,
            smtp_host: row.get(1)?,
            smtp_port: row.get::<_, i64>(2)? as u16,
            smtp_security: row.get(3)?,
            smtp_username: row.get(4)?,
            smtp_password,
            smtp_hello_domain: row.get(6)?,
            imap_host: row.get(7)?,
            imap_port: row.get::<_, i64>(8)? as u16,
            imap_security: row.get(9)?,
            imap_username: row.get(10)?,
            imap_password,
            imap_mailbox: row.get(12)?,
        }))
    }

    fn obfuscate_secret_if_needed(&self, secret: &str) -> Result<String, StoreError> {
        if secret.is_empty() {
            return Ok(String::new());
        }
        if SecretBox::is_obfuscated(secret) {
            return Ok(secret.to_owned());
        }
        secret_bridge::obfuscate(&self.key_path(), secret)
    }

    pub fn update_user_settings_field(
        &self,
        profile_id: &str,
        field: &str,
        value: &str,
    ) -> Result<(), StoreError> {
        match field {
            "nickname" => {
                self.connection().execute(
                    "UPDATE user_settings SET nickname = ?1 WHERE profile_id = ?2",
                    rusqlite::params![value, profile_id],
                )?;
            }
            "email_address" => {
                self.connection().execute(
                    "UPDATE user_settings SET email_address = ?1 WHERE profile_id = ?2",
                    rusqlite::params![value, profile_id],
                )?;
            }
            "avatar_url" => {
                let v: Option<&str> = if value.is_empty() { None } else { Some(value) };
                self.connection().execute(
                    "UPDATE user_settings SET avatar_url = ?1 WHERE profile_id = ?2",
                    rusqlite::params![v, profile_id],
                )?;
            }
            "setup_completed" => {
                let v: i64 = if value == "true" || value == "1" {
                    1
                } else {
                    0
                };
                self.connection().execute(
                    "UPDATE user_settings SET setup_completed = ?1 WHERE profile_id = ?2",
                    rusqlite::params![v, profile_id],
                )?;
            }
            "language" => {
                self.connection().execute(
                    "UPDATE user_settings SET language = ?1 WHERE profile_id = ?2",
                    rusqlite::params![value, profile_id],
                )?;
            }
            other => return Err(StoreError::InvalidColumn(other.to_owned())),
        }
        Ok(())
    }

    pub fn update_mail_settings_field(
        &self,
        profile_id: &str,
        field: &str,
        value: &str,
    ) -> Result<(), StoreError> {
        if field == "smtp.password" {
            return self.update_mail_password(profile_id, "smtp", value);
        }
        if field == "imap.password" {
            return self.update_mail_password(profile_id, "imap", value);
        }
        match field {
            "smtp.host" => self.update_mail_text_field(profile_id, "smtp_host", value)?,
            "smtp.port" => {
                let port: u16 = value
                    .parse()
                    .map_err(|_| StoreError::InvalidColumn(format!("{field}: неверный порт")))?;
                self.connection().execute(
                    "UPDATE mail_settings SET smtp_port = ?1 WHERE profile_id = ?2",
                    rusqlite::params![port as i64, profile_id],
                )?;
            }
            "smtp.security" => self.update_mail_text_field(profile_id, "smtp_security", value)?,
            "smtp.username" => self.update_mail_text_field(profile_id, "smtp_username", value)?,
            "smtp.hello_domain" => {
                self.update_mail_text_field(profile_id, "smtp_hello_domain", value)?
            }
            "imap.host" => self.update_mail_text_field(profile_id, "imap_host", value)?,
            "imap.port" => {
                let port: u16 = value
                    .parse()
                    .map_err(|_| StoreError::InvalidColumn(format!("{field}: неверный порт")))?;
                self.connection().execute(
                    "UPDATE mail_settings SET imap_port = ?1 WHERE profile_id = ?2",
                    rusqlite::params![port as i64, profile_id],
                )?;
            }
            "imap.security" => self.update_mail_text_field(profile_id, "imap_security", value)?,
            "imap.username" => self.update_mail_text_field(profile_id, "imap_username", value)?,
            "imap.mailbox" => self.update_mail_text_field(profile_id, "imap_mailbox", value)?,
            other => return Err(StoreError::InvalidColumn(other.to_owned())),
        }
        Ok(())
    }

    fn update_mail_text_field(
        &self,
        profile_id: &str,
        column: &str,
        value: &str,
    ) -> Result<(), StoreError> {
        self.connection().execute(
            &format!("UPDATE mail_settings SET {column} = ?1 WHERE profile_id = ?2"),
            rusqlite::params![value, profile_id],
        )?;
        Ok(())
    }

    fn update_mail_password(
        &self,
        profile_id: &str,
        which: &str,
        value: &str,
    ) -> Result<(), StoreError> {
        let column = match which {
            "smtp" => "smtp_password",
            "imap" => "imap_password",
            _ => return Err(StoreError::InvalidColumn(which.to_owned())),
        };
        let stored = self.obfuscate_secret_if_needed(value)?;
        self.connection().execute(
            &format!("UPDATE mail_settings SET {column} = ?1 WHERE profile_id = ?2"),
            rusqlite::params![stored, profile_id],
        )?;
        Ok(())
    }

    fn reveal_secret_with_lazy_migration(
        &self,
        profile_id: &str,
        column: &str,
        stored: &str,
    ) -> Result<String, StoreError> {
        if stored.is_empty() {
            return Ok(String::new());
        }
        if SecretBox::is_obfuscated(stored) {
            return secret_bridge::deobfuscate(&self.key_path(), stored);
        }

        let plaintext = stored.to_owned();
        let obfuscated = secret_bridge::obfuscate(&self.key_path(), &plaintext)?;
        self.connection().execute(
            &format!("UPDATE mail_settings SET {column} = ?1 WHERE profile_id = ?2"),
            params![obfuscated, profile_id],
        )?;
        Ok(plaintext)
    }
}

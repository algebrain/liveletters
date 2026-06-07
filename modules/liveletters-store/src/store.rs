use std::path::{Path, PathBuf};

use rusqlite::{Connection, Error as SqliteError};

use crate::StoreError;

pub(crate) const SECRET_BOX_KEY_FILENAME: &str = "mail-password-obfuscation.key";

pub struct Store {
    connection: Connection,
    data_dir: PathBuf,
}

impl Store {
    pub fn open_default() -> Result<Self, StoreError> {
        let paths = crate::StorePaths::from_environment()?;
        Self::open_at(paths.database_path())
    }

    pub fn open_for_home_dir(home_dir: impl AsRef<Path>) -> Result<Self, StoreError> {
        let paths = crate::StorePaths::for_home_dir(home_dir);
        Self::open_at(paths.database_path())
    }

    pub fn open_at(database_path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let database_path = database_path.as_ref();
        let data_dir = database_path
            .parent()
            .ok_or_else(|| StoreError::Io(std::io::Error::other("database path has no parent")))?
            .to_path_buf();
        if let Some(parent) = database_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let connection = Connection::open(database_path)?;
        let store = Self {
            connection,
            data_dir,
        };
        store.initialize_schema()?;
        Ok(store)
    }

    pub(crate) fn key_path(&self) -> PathBuf {
        self.data_dir.join(SECRET_BOX_KEY_FILENAME)
    }

    pub(crate) fn connection(&self) -> &Connection {
        &self.connection
    }

    fn initialize_schema(&self) -> Result<(), StoreError> {
        self.connection.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS posts (
                post_id TEXT PRIMARY KEY,
                resource_id TEXT NOT NULL,
                author_id TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                body TEXT NOT NULL,
                visibility TEXT NOT NULL,
                hidden INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS comments (
                comment_id TEXT PRIMARY KEY,
                post_id TEXT NOT NULL,
                parent_comment_id TEXT,
                author_id TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                body TEXT NOT NULL,
                visibility TEXT NOT NULL,
                hidden INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS outbox (
                event_id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                resource_id TEXT NOT NULL,
                delivery_json TEXT NOT NULL,
                message_body TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS raw_messages (
                message_id TEXT PRIMARY KEY,
                raw_message TEXT NOT NULL,
                status TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS raw_events (
                event_id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                resource_id TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                apply_status TEXT NOT NULL DEFAULT 'pending',
                failure_reason TEXT
            );

            CREATE TABLE IF NOT EXISTS deferred_events (
                event_id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                reason TEXT NOT NULL,
                payload_json TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS user_settings (
                profile_id TEXT PRIMARY KEY,
                nickname TEXT NOT NULL,
                email_address TEXT NOT NULL,
                avatar_url TEXT,
                setup_completed INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS subscriptions (
                resource_address TEXT NOT NULL,
                subscriber_delivery_address TEXT NOT NULL,
                PRIMARY KEY (resource_address, subscriber_delivery_address)
            );

            CREATE TABLE IF NOT EXISTS mail_settings (
                profile_id TEXT PRIMARY KEY,
                smtp_host TEXT NOT NULL,
                smtp_port INTEGER NOT NULL,
                smtp_security TEXT NOT NULL DEFAULT 'starttls',
                smtp_username TEXT NOT NULL,
                smtp_password TEXT NOT NULL,
                smtp_hello_domain TEXT NOT NULL,
                imap_host TEXT NOT NULL,
                imap_port INTEGER NOT NULL,
                imap_security TEXT NOT NULL DEFAULT 'starttls',
                imap_username TEXT NOT NULL,
                imap_password TEXT NOT NULL,
                imap_mailbox TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sync_cursors (
                profile_id TEXT PRIMARY KEY,
                last_imap_uid INTEGER NOT NULL
            );
            "#,
        )?;

        self.ensure_mail_settings_security_columns()?;
        self.ensure_subscriptions_use_delivery_address_key()?;

        Ok(())
    }

    fn ensure_mail_settings_security_columns(&self) -> Result<(), StoreError> {
        self.add_column_if_missing(
            "ALTER TABLE mail_settings ADD COLUMN smtp_security TEXT NOT NULL DEFAULT 'starttls'",
        )?;
        self.add_column_if_missing(
            "ALTER TABLE mail_settings ADD COLUMN imap_security TEXT NOT NULL DEFAULT 'starttls'",
        )?;
        Ok(())
    }

    fn add_column_if_missing(&self, sql: &str) -> Result<(), StoreError> {
        match self.connection.execute(sql, []) {
            Ok(_) => Ok(()),
            Err(SqliteError::SqliteFailure(_, Some(message)))
                if message.contains("duplicate column name") =>
            {
                Ok(())
            }
            Err(error) => Err(error.into()),
        }
    }

    fn ensure_subscriptions_use_delivery_address_key(&self) -> Result<(), StoreError> {
        if !self.table_has_column("subscriptions", "subscriber_account_id")? {
            return Ok(());
        }

        self.connection.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS subscriptions_new (
                resource_address TEXT NOT NULL,
                subscriber_delivery_address TEXT NOT NULL,
                PRIMARY KEY (resource_address, subscriber_delivery_address)
            );

            INSERT OR IGNORE INTO subscriptions_new
                (resource_address, subscriber_delivery_address)
            SELECT resource_address, subscriber_delivery_address
            FROM subscriptions
            WHERE subscriber_delivery_address <> '';

            DROP TABLE subscriptions;
            ALTER TABLE subscriptions_new RENAME TO subscriptions;
            "#,
        )?;

        Ok(())
    }

    fn table_has_column(&self, table: &str, column: &str) -> Result<bool, StoreError> {
        let mut stmt = self
            .connection
            .prepare(&format!("PRAGMA table_info({table})"))?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;

        for row in rows {
            if row? == column {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

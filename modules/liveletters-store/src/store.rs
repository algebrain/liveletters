use std::path::{Path, PathBuf};

use rusqlite::Connection;

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
        connection.execute_batch("PRAGMA foreign_keys = ON;")?;
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
            CREATE TABLE IF NOT EXISTS authors (
                email         TEXT NOT NULL PRIMARY KEY,
                nickname      TEXT NOT NULL,
                source        TEXT NOT NULL,
                first_seen_at INTEGER NOT NULL,
                updated_at    INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_authors_nickname ON authors(nickname);

            CREATE TABLE IF NOT EXISTS user_settings (
                profile_id       TEXT PRIMARY KEY,
                author_email     TEXT NOT NULL UNIQUE REFERENCES authors(email),
                avatar_url       TEXT,
                language         TEXT NOT NULL DEFAULT 'ru',
                setup_completed  INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS mail_settings (
                profile_id            TEXT PRIMARY KEY,
                smtp_host             TEXT NOT NULL,
                smtp_port             INTEGER NOT NULL,
                smtp_security         TEXT NOT NULL DEFAULT 'starttls',
                smtp_username         TEXT NOT NULL,
                smtp_password         TEXT NOT NULL,
                smtp_hello_domain     TEXT NOT NULL,
                imap_host             TEXT NOT NULL,
                imap_port             INTEGER NOT NULL,
                imap_security         TEXT NOT NULL DEFAULT 'starttls',
                imap_username         TEXT NOT NULL,
                imap_password         TEXT NOT NULL,
                imap_mailbox          TEXT NOT NULL,
                initial_lookback_days INTEGER NOT NULL DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS subscriptions (
                resource_email   TEXT NOT NULL REFERENCES authors(email),
                subscriber_email TEXT NOT NULL REFERENCES authors(email),
                PRIMARY KEY (resource_email, subscriber_email)
            );

            CREATE TABLE IF NOT EXISTS local_subscriptions (
                profile_id       TEXT NOT NULL,
                resource_email   TEXT NOT NULL REFERENCES authors(email),
                PRIMARY KEY (profile_id, resource_email)
            );

            CREATE TABLE IF NOT EXISTS pending_subscriptions (
                profile_id       TEXT NOT NULL,
                resource_email   TEXT NOT NULL REFERENCES authors(email),
                requested_at     INTEGER NOT NULL,
                last_attempt_at  INTEGER NOT NULL,
                PRIMARY KEY (profile_id, resource_email)
            );

            CREATE TABLE IF NOT EXISTS friends (
                owner_resource_email TEXT NOT NULL REFERENCES authors(email),
                friend_email         TEXT NOT NULL REFERENCES authors(email),
                PRIMARY KEY (owner_resource_email, friend_email)
            );

            CREATE TABLE IF NOT EXISTS pending_friends (
                profile_id                TEXT NOT NULL,
                owner_resource_email      TEXT NOT NULL REFERENCES authors(email),
                friend_email              TEXT NOT NULL REFERENCES authors(email),
                subscribed_resource_email TEXT NOT NULL REFERENCES authors(email),
                requested_at              INTEGER NOT NULL,
                last_attempt_at           INTEGER NOT NULL,
                PRIMARY KEY (profile_id, owner_resource_email, friend_email)
            );

            CREATE TABLE IF NOT EXISTS friend_of (
                profile_id     TEXT NOT NULL,
                resource_email TEXT NOT NULL REFERENCES authors(email),
                PRIMARY KEY (profile_id, resource_email)
            );

            CREATE TABLE IF NOT EXISTS resources_owned (
                profile_id       TEXT NOT NULL,
                resource_email   TEXT NOT NULL REFERENCES authors(email),
                PRIMARY KEY (profile_id, resource_email)
            );

            CREATE TABLE IF NOT EXISTS posts (
                post_id        TEXT PRIMARY KEY,
                resource_email TEXT NOT NULL REFERENCES authors(email),
                author_email   TEXT NOT NULL REFERENCES authors(email),
                created_at     INTEGER NOT NULL,
                body           TEXT NOT NULL,
                visibility     TEXT NOT NULL,
                hidden         INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS comments (
                comment_id          TEXT PRIMARY KEY,
                post_id             TEXT NOT NULL REFERENCES posts(post_id),
                parent_comment_id   TEXT REFERENCES comments(comment_id),
                author_email        TEXT NOT NULL REFERENCES authors(email),
                created_at          INTEGER NOT NULL,
                body                TEXT NOT NULL,
                visibility          TEXT NOT NULL,
                hidden              INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS outbox (
                event_id             TEXT PRIMARY KEY,
                event_type           TEXT NOT NULL,
                author_email         TEXT NOT NULL REFERENCES authors(email),
                resource_email       TEXT REFERENCES authors(email),
                delivery_json        TEXT NOT NULL,
                message_body         TEXT NOT NULL,
                message_id           TEXT,
                subject              TEXT,
                human_readable_body  TEXT
            );

            CREATE TABLE IF NOT EXISTS bounce_records (
                original_message_id   TEXT PRIMARY KEY,
                event_id              TEXT,
                final_recipient_email TEXT REFERENCES authors(email),
                status_code           TEXT,
                diagnostic_code       TEXT,
                received_at           INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS raw_messages (
                message_id   TEXT PRIMARY KEY,
                raw_message  TEXT NOT NULL,
                status       TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS raw_events (
                event_id      TEXT PRIMARY KEY,
                event_type    TEXT NOT NULL,
                resource_id   TEXT NOT NULL,
                payload_json  TEXT NOT NULL,
                apply_status  TEXT NOT NULL DEFAULT 'pending',
                failure_reason TEXT
            );

            CREATE TABLE IF NOT EXISTS deferred_events (
                event_id      TEXT PRIMARY KEY,
                event_type    TEXT NOT NULL,
                reason        TEXT NOT NULL,
                payload_json  TEXT NOT NULL,
                origin        TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sync_cursors (
                profile_id     TEXT PRIMARY KEY,
                last_imap_uid  INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS receive_addresses (
                profile_id  TEXT NOT NULL,
                address     TEXT NOT NULL,
                PRIMARY KEY (profile_id, address)
            );
            "#,
        )?;

        Ok(())
    }
}

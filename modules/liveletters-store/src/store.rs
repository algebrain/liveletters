use std::{fs, path::Path};

use rusqlite::{params, Connection};

use crate::{
    CommentRecord, DeferredEventRecord, MailSettingsRecord, OutboxRecord, PostRecord,
    RawEventRecord, RawMessageRecord, StoreError, StorePaths, UserSettingsRecord,
};

pub struct Store {
    connection: Connection,
}

impl Store {
    pub fn open_default() -> Result<Self, StoreError> {
        let paths = StorePaths::from_environment()?;
        Self::open_at(paths.database_path())
    }

    pub fn open_in_memory() -> Result<Self, StoreError> {
        let connection = Connection::open_in_memory()?;
        let store = Self { connection };
        store.initialize_schema()?;
        Ok(store)
    }

    pub fn open_at(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let connection = Connection::open(path)?;
        let store = Self { connection };
        store.initialize_schema()?;
        Ok(store)
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

            CREATE TABLE IF NOT EXISTS mail_settings (
                profile_id TEXT PRIMARY KEY,
                smtp_host TEXT NOT NULL,
                smtp_port INTEGER NOT NULL,
                smtp_username TEXT NOT NULL,
                smtp_password TEXT NOT NULL,
                smtp_hello_domain TEXT NOT NULL,
                imap_host TEXT NOT NULL,
                imap_port INTEGER NOT NULL,
                imap_username TEXT NOT NULL,
                imap_password TEXT NOT NULL,
                imap_mailbox TEXT NOT NULL
            );
            "#,
        )?;

        Ok(())
    }

    pub fn save_post_record(&self, post: &PostRecord) -> Result<(), StoreError> {
        self.connection.execute(
            r#"
            INSERT OR REPLACE INTO posts
                (post_id, resource_id, author_id, created_at, body, visibility, hidden)
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                post.post_id,
                post.resource_id,
                post.author_id,
                post.created_at as i64,
                post.body,
                post.visibility,
                post.hidden as i64,
            ],
        )?;

        Ok(())
    }

    pub fn save_comment_record(&self, comment: &CommentRecord) -> Result<(), StoreError> {
        self.connection.execute(
            r#"
            INSERT OR REPLACE INTO comments
                (comment_id, post_id, parent_comment_id, author_id, created_at, body, visibility, hidden)
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                comment.comment_id,
                comment.post_id,
                comment.parent_comment_id,
                comment.author_id,
                comment.created_at as i64,
                comment.body,
                comment.visibility,
                comment.hidden as i64,
            ],
        )?;

        Ok(())
    }

    pub fn list_posts(&self) -> Result<Vec<PostRecord>, StoreError> {
        let mut stmt = self.connection.prepare(
            r#"
            SELECT post_id, resource_id, author_id, created_at, body, visibility, hidden
            FROM posts
            ORDER BY created_at ASC, post_id ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(PostRecord {
                post_id: row.get(0)?,
                resource_id: row.get(1)?,
                author_id: row.get(2)?,
                created_at: row.get::<_, i64>(3)? as u64,
                body: row.get(4)?,
                visibility: row.get(5)?,
                hidden: row.get::<_, i64>(6)? != 0,
            })
        })?;

        let mut posts = Vec::new();
        for row in rows {
            posts.push(row?);
        }

        Ok(posts)
    }

    pub fn get_post_record(&self, post_id: &str) -> Result<Option<PostRecord>, StoreError> {
        let mut stmt = self.connection.prepare(
            r#"
            SELECT post_id, resource_id, author_id, created_at, body, visibility, hidden
            FROM posts
            WHERE post_id = ?1
            "#,
        )?;

        let mut rows = stmt.query([post_id])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };

        Ok(Some(PostRecord {
            post_id: row.get(0)?,
            resource_id: row.get(1)?,
            author_id: row.get(2)?,
            created_at: row.get::<_, i64>(3)? as u64,
            body: row.get(4)?,
            visibility: row.get(5)?,
            hidden: row.get::<_, i64>(6)? != 0,
        }))
    }

    pub fn list_comments_for_post(&self, post_id: &str) -> Result<Vec<CommentRecord>, StoreError> {
        let mut stmt = self.connection.prepare(
            r#"
            SELECT comment_id, post_id, parent_comment_id, author_id, created_at, body, visibility, hidden
            FROM comments
            WHERE post_id = ?1
            ORDER BY created_at ASC, comment_id ASC
            "#,
        )?;

        let rows = stmt.query_map([post_id], |row| {
            Ok(CommentRecord {
                comment_id: row.get(0)?,
                post_id: row.get(1)?,
                parent_comment_id: row.get(2)?,
                author_id: row.get(3)?,
                created_at: row.get::<_, i64>(4)? as u64,
                body: row.get(5)?,
                visibility: row.get(6)?,
                hidden: row.get::<_, i64>(7)? != 0,
            })
        })?;

        let mut comments = Vec::new();
        for row in rows {
            comments.push(row?);
        }

        Ok(comments)
    }

    pub fn get_comment_record(&self, comment_id: &str) -> Result<Option<CommentRecord>, StoreError> {
        let mut stmt = self.connection.prepare(
            r#"
            SELECT comment_id, post_id, parent_comment_id, author_id, created_at, body, visibility, hidden
            FROM comments
            WHERE comment_id = ?1
            "#,
        )?;

        let mut rows = stmt.query([comment_id])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };

        Ok(Some(CommentRecord {
            comment_id: row.get(0)?,
            post_id: row.get(1)?,
            parent_comment_id: row.get(2)?,
            author_id: row.get(3)?,
            created_at: row.get::<_, i64>(4)? as u64,
            body: row.get(5)?,
            visibility: row.get(6)?,
            hidden: row.get::<_, i64>(7)? != 0,
        }))
    }

    pub fn save_outbox_record(&self, record: &OutboxRecord) -> Result<(), StoreError> {
        self.connection.execute(
            r#"
            INSERT OR REPLACE INTO outbox
                (event_id, event_type, resource_id, message_body)
            VALUES
                (?1, ?2, ?3, ?4)
            "#,
            params![
                record.event_id,
                record.event_type,
                record.resource_id,
                record.message_body,
            ],
        )?;

        Ok(())
    }

    pub fn list_outbox_records(&self) -> Result<Vec<OutboxRecord>, StoreError> {
        let mut stmt = self.connection.prepare(
            r#"
            SELECT event_id, event_type, resource_id, message_body
            FROM outbox
            ORDER BY rowid ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(OutboxRecord {
                event_id: row.get(0)?,
                event_type: row.get(1)?,
                resource_id: row.get(2)?,
                message_body: row.get(3)?,
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }

        Ok(records)
    }

    pub fn save_raw_message_record(&self, record: &RawMessageRecord) -> Result<(), StoreError> {
        self.connection.execute(
            r#"
            INSERT OR REPLACE INTO raw_messages
                (message_id, raw_message, status)
            VALUES
                (?1, ?2, ?3)
            "#,
            params![record.message_id, record.raw_message, record.status],
        )?;

        Ok(())
    }

    pub fn list_raw_message_records(&self) -> Result<Vec<RawMessageRecord>, StoreError> {
        let mut stmt = self.connection.prepare(
            r#"
            SELECT message_id, raw_message, status
            FROM raw_messages
            ORDER BY rowid ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(RawMessageRecord {
                message_id: row.get(0)?,
                raw_message: row.get(1)?,
                status: row.get(2)?,
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }

        Ok(records)
    }

    pub fn save_raw_event_record(&self, record: &RawEventRecord) -> Result<(), StoreError> {
        self.connection.execute(
            r#"
            INSERT OR REPLACE INTO raw_events
                (event_id, event_type, resource_id, payload_json, apply_status, failure_reason)
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                record.event_id,
                record.event_type,
                record.resource_id,
                record.payload_json,
                record.apply_status,
                record.failure_reason,
            ],
        )?;

        Ok(())
    }

    pub fn list_raw_event_records(&self) -> Result<Vec<RawEventRecord>, StoreError> {
        let mut stmt = self.connection.prepare(
            r#"
            SELECT event_id, event_type, resource_id, payload_json, apply_status, failure_reason
            FROM raw_events
            ORDER BY rowid ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(RawEventRecord {
                event_id: row.get(0)?,
                event_type: row.get(1)?,
                resource_id: row.get(2)?,
                payload_json: row.get(3)?,
                apply_status: row.get(4)?,
                failure_reason: row.get(5)?,
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }

        Ok(records)
    }

    pub fn has_raw_event(&self, event_id: &str) -> Result<bool, StoreError> {
        let mut stmt = self.connection.prepare(
            "SELECT 1 FROM raw_events WHERE event_id = ?1 LIMIT 1",
        )?;
        let mut rows = stmt.query([event_id])?;
        Ok(rows.next()?.is_some())
    }

    pub fn save_deferred_event_record(
        &self,
        record: &DeferredEventRecord,
    ) -> Result<(), StoreError> {
        self.connection.execute(
            r#"
            INSERT OR REPLACE INTO deferred_events
                (event_id, event_type, reason, payload_json)
            VALUES
                (?1, ?2, ?3, ?4)
            "#,
            params![
                record.event_id,
                record.event_type,
                record.reason,
                record.payload_json,
            ],
        )?;

        Ok(())
    }

    pub fn list_deferred_event_records(&self) -> Result<Vec<DeferredEventRecord>, StoreError> {
        let mut stmt = self.connection.prepare(
            r#"
            SELECT event_id, event_type, reason, payload_json
            FROM deferred_events
            ORDER BY rowid ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DeferredEventRecord {
                event_id: row.get(0)?,
                event_type: row.get(1)?,
                reason: row.get(2)?,
                payload_json: row.get(3)?,
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }

        Ok(records)
    }

    pub fn delete_deferred_event_record(&self, event_id: &str) -> Result<(), StoreError> {
        self.connection.execute(
            "DELETE FROM deferred_events WHERE event_id = ?1",
            [event_id],
        )?;

        Ok(())
    }

    pub fn save_user_settings_record(
        &self,
        record: &UserSettingsRecord,
    ) -> Result<(), StoreError> {
        self.connection.execute(
            r#"
            INSERT OR REPLACE INTO user_settings
                (profile_id, nickname, email_address, avatar_url, setup_completed)
            VALUES
                (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                record.profile_id,
                record.nickname,
                record.email_address,
                record.avatar_url,
                record.setup_completed as i64,
            ],
        )?;

        Ok(())
    }

    pub fn get_user_settings_record(
        &self,
        profile_id: &str,
    ) -> Result<Option<UserSettingsRecord>, StoreError> {
        let mut stmt = self.connection.prepare(
            r#"
            SELECT profile_id, nickname, email_address, avatar_url, setup_completed
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
            setup_completed: row.get::<_, i64>(4)? != 0,
        }))
    }

    pub fn save_mail_settings_record(
        &self,
        record: &MailSettingsRecord,
    ) -> Result<(), StoreError> {
        self.connection.execute(
            r#"
            INSERT OR REPLACE INTO mail_settings
                (
                    profile_id,
                    smtp_host,
                    smtp_port,
                    smtp_username,
                    smtp_password,
                    smtp_hello_domain,
                    imap_host,
                    imap_port,
                    imap_username,
                    imap_password,
                    imap_mailbox
                )
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                record.profile_id,
                record.smtp_host,
                record.smtp_port as i64,
                record.smtp_username,
                record.smtp_password,
                record.smtp_hello_domain,
                record.imap_host,
                record.imap_port as i64,
                record.imap_username,
                record.imap_password,
                record.imap_mailbox,
            ],
        )?;

        Ok(())
    }

    pub fn get_mail_settings_record(
        &self,
        profile_id: &str,
    ) -> Result<Option<MailSettingsRecord>, StoreError> {
        let mut stmt = self.connection.prepare(
            r#"
            SELECT
                profile_id,
                smtp_host,
                smtp_port,
                smtp_username,
                smtp_password,
                smtp_hello_domain,
                imap_host,
                imap_port,
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

        Ok(Some(MailSettingsRecord {
            profile_id: row.get(0)?,
            smtp_host: row.get(1)?,
            smtp_port: row.get::<_, i64>(2)? as u16,
            smtp_username: row.get(3)?,
            smtp_password: row.get(4)?,
            smtp_hello_domain: row.get(5)?,
            imap_host: row.get(6)?,
            imap_port: row.get::<_, i64>(7)? as u16,
            imap_username: row.get(8)?,
            imap_password: row.get(9)?,
            imap_mailbox: row.get(10)?,
        }))
    }
}

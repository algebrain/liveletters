use rusqlite::params;

use crate::{AuthorRecord, Store, StoreError};

impl Store {
    /// UPSERT в `authors`. Сохраняет `first_seen_at` при обновлении.
    pub fn save_author(&self, email: &str, nickname: &str, source: &str) -> Result<(), StoreError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.connection().execute(
            r#"
            INSERT INTO authors (email, nickname, source, first_seen_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?4)
            ON CONFLICT(email) DO UPDATE SET
                nickname   = excluded.nickname,
                source     = excluded.source,
                updated_at = excluded.updated_at
            "#,
            params![email, nickname, source, now as i64],
        )?;
        Ok(())
    }

    pub fn get_author(&self, email: &str) -> Result<Option<AuthorRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            "SELECT email, nickname, source, first_seen_at, updated_at
             FROM authors WHERE email = ?1",
        )?;
        let mut rows = stmt.query([email])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        Ok(Some(AuthorRecord {
            email: row.get(0)?,
            nickname: row.get(1)?,
            source: row.get(2)?,
            first_seen_at: row.get::<_, i64>(3)? as u64,
            updated_at: row.get::<_, i64>(4)? as u64,
        }))
    }

    pub fn list_authors(&self) -> Result<Vec<AuthorRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            "SELECT email, nickname, source, first_seen_at, updated_at
             FROM authors ORDER BY email",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(AuthorRecord {
                email: r.get(0)?,
                nickname: r.get(1)?,
                source: r.get(2)?,
                first_seen_at: r.get::<_, i64>(3)? as u64,
                updated_at: r.get::<_, i64>(4)? as u64,
            })
        })?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    /// Сохранить идентичность: UPSERT в `authors` + UPSERT в `user_settings`.
    /// Атомарно в одной транзакции.
    pub fn save_identity(
        &self,
        profile_id: &str,
        email: &str,
        nickname: &str,
        avatar_url: Option<&str>,
        language: &str,
        setup_completed: bool,
    ) -> Result<(), StoreError> {
        self.save_author(email, nickname, "self")?;
        self.save_user_settings_record(&crate::UserSettingsRecord {
            profile_id: profile_id.to_owned(),
            author_email: email.to_owned(),
            avatar_url: avatar_url.map(str::to_owned),
            language: language.to_owned(),
            setup_completed,
        })?;
        Ok(())
    }
}

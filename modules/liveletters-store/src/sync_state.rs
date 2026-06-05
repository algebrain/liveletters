use rusqlite::params;

use crate::{Store, StoreError};

impl Store {
    pub fn get_sync_cursor(&self, profile_id: &str) -> Result<Option<u64>, StoreError> {
        let mut stmt = self
            .connection()
            .prepare("SELECT last_imap_uid FROM sync_cursors WHERE profile_id = ?1")?;
        let mut rows = stmt.query(params![profile_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    pub fn save_sync_cursor(&self, profile_id: &str, uid: u64) -> Result<(), StoreError> {
        self.connection().execute(
            r#"
            INSERT INTO sync_cursors (profile_id, last_imap_uid)
            VALUES (?1, ?2)
            ON CONFLICT(profile_id) DO UPDATE SET last_imap_uid = excluded.last_imap_uid
            "#,
            params![profile_id, uid],
        )?;
        Ok(())
    }
}

use rusqlite::params;

use crate::{OutboxRecord, Store, StoreError};

impl Store {
    pub fn save_outbox_record(&self, record: &OutboxRecord) -> Result<(), StoreError> {
        self.connection().execute(
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
        let mut stmt = self.connection().prepare(
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

    pub fn count_outbox(&self) -> Result<u64, StoreError> {
        let n: i64 = self
            .connection()
            .query_row("SELECT COUNT(*) FROM outbox", [], |row| row.get(0))?;
        Ok(n as u64)
    }

    pub fn delete_outbox_record(&self, event_id: &str) -> Result<bool, StoreError> {
        let changed = self
            .connection()
            .execute("DELETE FROM outbox WHERE event_id = ?1", params![event_id])?;
        Ok(changed > 0)
    }
}

use rusqlite::params;

use crate::{BounceRecord, Store, StoreError};

impl Store {
    pub fn save_bounce_record(&self, record: &BounceRecord) -> Result<(), StoreError> {
        self.connection().execute(
            r#"
            INSERT OR REPLACE INTO bounce_records
                (original_message_id, event_id, final_recipient_email,
                 status_code, diagnostic_code, received_at)
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                record.original_message_id,
                record.event_id,
                record.final_recipient_email,
                record.status_code,
                record.diagnostic_code,
                record.received_at as i64,
            ],
        )?;
        Ok(())
    }

    pub fn list_bounce_records(&self) -> Result<Vec<BounceRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            "SELECT original_message_id, event_id, final_recipient_email,
                    status_code, diagnostic_code, received_at
             FROM bounce_records ORDER BY received_at",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(BounceRecord {
                original_message_id: r.get(0)?,
                event_id: r.get(1)?,
                final_recipient_email: r.get(2)?,
                status_code: r.get(3)?,
                diagnostic_code: r.get(4)?,
                received_at: r.get::<_, i64>(5)? as u64,
            })
        })?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn find_bounce_by_message_id(
        &self,
        message_id: &str,
    ) -> Result<Option<BounceRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            "SELECT original_message_id, event_id, final_recipient_email,
                    status_code, diagnostic_code, received_at
             FROM bounce_records WHERE original_message_id = ?1",
        )?;
        let mut rows = stmt.query(params![message_id])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        Ok(Some(BounceRecord {
            original_message_id: row.get(0)?,
            event_id: row.get(1)?,
            final_recipient_email: row.get(2)?,
            status_code: row.get(3)?,
            diagnostic_code: row.get(4)?,
            received_at: row.get::<_, i64>(5)? as u64,
        }))
    }
}

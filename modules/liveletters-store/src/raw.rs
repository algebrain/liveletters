use rusqlite::params;

use crate::{DeferredEventRecord, RawEventRecord, RawMessageRecord, Store, StoreError};

impl Store {
    pub fn save_raw_message_record(&self, record: &RawMessageRecord) -> Result<(), StoreError> {
        self.connection().execute(
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
        let mut stmt = self.connection().prepare(
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

    pub fn get_raw_message_record(
        &self,
        message_id: &str,
    ) -> Result<Option<RawMessageRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            "SELECT message_id, raw_message, status FROM raw_messages WHERE message_id = ?1",
        )?;
        let mut rows = stmt.query([message_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(RawMessageRecord {
                message_id: row.get(0)?,
                raw_message: row.get(1)?,
                status: row.get(2)?,
            })),
            None => Ok(None),
        }
    }

    pub fn list_raw_message_records_paged(
        &self,
        status: Option<&str>,
        limit: usize,
    ) -> Result<Vec<RawMessageRecord>, StoreError> {
        let conn = self.connection();
        let records = match status {
            Some(s) => {
                let mut stmt = conn.prepare(
                    r#"
                    SELECT message_id, raw_message, status
                    FROM raw_messages
                    WHERE status = ?1
                    ORDER BY rowid DESC
                    LIMIT ?2
                    "#,
                )?;
                let rows = stmt.query_map([s, &limit.to_string()], |row| {
                    Ok(RawMessageRecord {
                        message_id: row.get(0)?,
                        raw_message: row.get(1)?,
                        status: row.get(2)?,
                    })
                })?;
                let mut v = Vec::new();
                for row in rows {
                    v.push(row?);
                }
                v
            }
            None => {
                let mut stmt = conn.prepare(
                    r#"
                    SELECT message_id, raw_message, status
                    FROM raw_messages
                    ORDER BY rowid DESC
                    LIMIT ?1
                    "#,
                )?;
                let rows = stmt.query_map([&limit.to_string()], |row| {
                    Ok(RawMessageRecord {
                        message_id: row.get(0)?,
                        raw_message: row.get(1)?,
                        status: row.get(2)?,
                    })
                })?;
                let mut v = Vec::new();
                for row in rows {
                    v.push(row?);
                }
                v
            }
        };
        Ok(records)
    }

    pub fn save_raw_event_record(&self, record: &RawEventRecord) -> Result<(), StoreError> {
        self.connection().execute(
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
        let mut stmt = self.connection().prepare(
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
        let mut stmt = self
            .connection()
            .prepare("SELECT 1 FROM raw_events WHERE event_id = ?1 LIMIT 1")?;
        let mut rows = stmt.query([event_id])?;
        Ok(rows.next()?.is_some())
    }

    pub fn save_deferred_event_record(
        &self,
        record: &DeferredEventRecord,
    ) -> Result<(), StoreError> {
        self.connection().execute(
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
        let mut stmt = self.connection().prepare(
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

    pub fn list_deferred_events(
        &self,
        limit: usize,
    ) -> Result<Vec<DeferredEventRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            r#"
            SELECT event_id, event_type, reason, payload_json
            FROM deferred_events
            ORDER BY rowid DESC
            LIMIT ?1
            "#,
        )?;

        let rows = stmt.query_map([&limit.to_string()], |row| {
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

    pub fn count_deferred_events(&self) -> Result<u64, StoreError> {
        let n: i64 =
            self.connection()
                .query_row("SELECT COUNT(*) FROM deferred_events", [], |row| row.get(0))?;
        Ok(n as u64)
    }

    pub fn delete_deferred_event_record(&self, event_id: &str) -> Result<(), StoreError> {
        self.connection().execute(
            "DELETE FROM deferred_events WHERE event_id = ?1",
            [event_id],
        )?;

        Ok(())
    }
}

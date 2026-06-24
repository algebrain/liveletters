use rusqlite::params;

use crate::{DeferredEventRecord, RawEventRecord, RawMessageRecord, Store, StoreError};
use liveletters_utils::time::unix_now;

/// Статусы сырых писем, подлежащие регулярной чистке (мусорные). Всё, что
/// представляет осмысленную историю (`applied`, `duplicate`, `bounce.*`),
/// чисткой не затрагивается.
const CLEANABLE_STATUSES: &[&str] = &["malformed", "invalid", "rate_limited"];

impl Store {
    pub fn save_raw_message_record(&self, record: &RawMessageRecord) -> Result<(), StoreError> {
        self.connection().execute(
            r#"
            INSERT OR REPLACE INTO raw_messages
                (message_id, raw_message, status, received_at)
            VALUES
                (?1, ?2, ?3, ?4)
            "#,
            params![
                record.message_id,
                record.raw_message,
                record.status,
                record.received_at as i64,
            ],
        )?;

        Ok(())
    }

    pub fn list_raw_message_records(&self) -> Result<Vec<RawMessageRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            r#"
            SELECT message_id, raw_message, status, received_at
            FROM raw_messages
            ORDER BY rowid ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(RawMessageRecord {
                message_id: row.get(0)?,
                raw_message: row.get(1)?,
                status: row.get(2)?,
                received_at: row.get::<_, i64>(3)? as u64,
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
            "SELECT message_id, raw_message, status, received_at FROM raw_messages WHERE message_id = ?1",
        )?;
        let mut rows = stmt.query([message_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(RawMessageRecord {
                message_id: row.get(0)?,
                raw_message: row.get(1)?,
                status: row.get(2)?,
                received_at: row.get::<_, i64>(3)? as u64,
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
                    SELECT message_id, raw_message, status, received_at
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
                        received_at: row.get::<_, i64>(3)? as u64,
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
                    SELECT message_id, raw_message, status, received_at
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
                        received_at: row.get::<_, i64>(3)? as u64,
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

    pub fn count_raw_messages(&self) -> Result<u64, StoreError> {
        let n: i64 =
            self.connection()
                .query_row("SELECT COUNT(*) FROM raw_messages", [], |row| row.get(0))?;
        Ok(n as u64)
    }

    pub fn count_raw_messages_by_status(&self, status: &str) -> Result<u64, StoreError> {
        let n: i64 = self.connection().query_row(
            "SELECT COUNT(*) FROM raw_messages WHERE status = ?1",
            params![status],
            |row| row.get(0),
        )?;
        Ok(n as u64)
    }

    /// Удалить сырые письма с «мусорными» статусами (`malformed`, `invalid`,
    /// `rate_limited`) старше `days` суток. Возвращает число удалённых строк.
    pub fn cleanup_old_raw_messages(&self, days: u32) -> Result<u64, StoreError> {
        let now = unix_now();
        let cutoff = now.saturating_sub((days as u64) * 86_400);
        self.cleanup_raw_messages_before(cutoff)
    }

    /// Удалить сырые письма с «мусорными» статусами, у которых `received_at`
    /// строго меньше `threshold_unix`. Низкоуровневая основа для
    /// [`Self::cleanup_old_raw_messages`]; удобна для детерминированных тестов.
    pub fn cleanup_raw_messages_before(&self, threshold_unix: u64) -> Result<u64, StoreError> {
        let placeholders = CLEANABLE_STATUSES
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "DELETE FROM raw_messages WHERE status IN ({placeholders}) AND received_at < ?"
        );
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = CLEANABLE_STATUSES
            .iter()
            .map(|s| Box::new((*s).to_owned()) as Box<dyn rusqlite::ToSql>)
            .collect();
        params_vec.push(Box::new(threshold_unix as i64));
        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|b| b.as_ref()).collect();
        let deleted = self.connection().execute(&sql, params_refs.as_slice())?;
        Ok(deleted as u64)
    }

    /// Удалить самые старые «мусорные» сырые письма, пока их суммарное число не
    /// станет не больше `max_rows`. Возвращает число удалённых строк.
    pub fn enforce_raw_messages_quota(&self, max_rows: usize) -> Result<u64, StoreError> {
        let placeholders = CLEANABLE_STATUSES
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let count_sql =
            format!("SELECT COUNT(*) FROM raw_messages WHERE status IN ({placeholders})");
        let params_vec: Vec<Box<dyn rusqlite::ToSql>> = CLEANABLE_STATUSES
            .iter()
            .map(|s| Box::new((*s).to_owned()) as Box<dyn rusqlite::ToSql>)
            .collect();
        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|b| b.as_ref()).collect();
        let total: i64 =
            self.connection()
                .query_row(&count_sql, params_refs.as_slice(), |row| row.get(0))?;
        let excess = (total as usize).saturating_sub(max_rows);
        if excess == 0 {
            return Ok(0);
        }

        let delete_sql = format!(
            "DELETE FROM raw_messages WHERE message_id IN ( \
               SELECT message_id FROM raw_messages \
               WHERE status IN ({placeholders}) \
               ORDER BY received_at ASC, rowid ASC \
               LIMIT ? \
             )"
        );
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = CLEANABLE_STATUSES
            .iter()
            .map(|s| Box::new((*s).to_owned()) as Box<dyn rusqlite::ToSql>)
            .collect();
        params_vec.push(Box::new(excess as i64));
        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|b| b.as_ref()).collect();
        let deleted = self
            .connection()
            .execute(&delete_sql, params_refs.as_slice())?;
        Ok(deleted as u64)
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
                (event_id, event_type, reason, payload_json, origin)
            VALUES
                (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                record.event_id,
                record.event_type,
                record.reason,
                record.payload_json,
                record.origin,
            ],
        )?;

        Ok(())
    }

    pub fn list_deferred_event_records(&self) -> Result<Vec<DeferredEventRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            r#"
            SELECT event_id, event_type, reason, payload_json, origin
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
                origin: row.get(4)?,
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
            SELECT event_id, event_type, reason, payload_json, origin
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
                origin: row.get(4)?,
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

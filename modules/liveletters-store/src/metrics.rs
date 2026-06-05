use crate::{Store, StoreError};

const ALLOWED_TABLES: &[&str] = &[
    "posts",
    "comments",
    "outbox",
    "raw_messages",
    "deferred_events",
    "subscriptions",
    "user_settings",
    "mail_settings",
    "identity_subscriptions",
];

impl Store {
    /// Размер таблицы в байтах через виртуальную таблицу `dbstat`
    /// (`SUM(pgsize) WHERE name = '...'`).
    ///
    /// `table` обязан быть в `ALLOWED_TABLES` — иначе
    /// `StoreError::InvalidTable` (защита от SQL-инъекции в `format!`).
    /// На пустой таблице возвращает `0`.
    pub fn table_size(&self, table: &str) -> Result<u64, StoreError> {
        if !ALLOWED_TABLES.contains(&table) {
            return Err(StoreError::InvalidTable(table.to_owned()));
        }
        let conn = self.connection();
        let sql = format!("SELECT COALESCE(SUM(pgsize), 0) FROM dbstat WHERE name = '{table}'");
        let bytes: i64 = conn.query_row(&sql, [], |r| r.get(0))?;
        Ok(bytes.max(0) as u64)
    }
}

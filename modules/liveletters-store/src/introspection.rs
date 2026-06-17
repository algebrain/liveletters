use rusqlite::Connection;

use crate::Store;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnInfo {
    pub name: String,
    pub column_type: String,
    pub nullable: bool,
    pub default: Option<String>,
    pub pk: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForeignKeyInfo {
    pub from: Vec<String>,
    pub to: Vec<String>,
    pub table: String,
}

impl Store {
    /// Список имён таблиц в БД (без системных).
    pub fn list_table_names(&self) -> Result<Vec<String>, crate::StoreError> {
        let conn = self.connection();
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut names = Vec::new();
        for row in rows {
            names.push(row?);
        }
        Ok(names)
    }

    /// Колонки таблицы (через `PRAGMA table_info`).
    /// Возвращает пустой список, если таблицы нет.
    pub fn table_columns(&self, table: &str) -> Result<Vec<ColumnInfo>, crate::StoreError> {
        let conn = self.connection();
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
        let rows = stmt.query_map([], |row| {
            Ok(ColumnInfo {
                name: row.get(1)?,
                column_type: row.get(2)?,
                nullable: row.get::<_, i64>(3)? == 0,
                default: row.get(4)?,
                pk: row.get::<_, i64>(5)? != 0,
            })
        })?;
        let mut cols = Vec::new();
        for row in rows {
            cols.push(row?);
        }
        Ok(cols)
    }

    /// Внешние ключи таблицы (через `PRAGMA foreign_key_list`).
    pub fn foreign_keys(&self, table: &str) -> Result<Vec<ForeignKeyInfo>, crate::StoreError> {
        let conn = self.connection();
        let mut stmt = conn.prepare(&format!("PRAGMA foreign_key_list({table})"))?;
        let rows = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let seq: i64 = row.get(1)?;
            let target_table: String = row.get(2)?;
            let from: String = row.get(3)?;
            let to: String = row.get(4)?;
            Ok((id, seq, target_table, from, to))
        })?;
        let mut grouped: std::collections::BTreeMap<i64, ForeignKeyInfo> =
            std::collections::BTreeMap::new();
        for row in rows {
            let (id, seq, target_table, from, to) = row?;
            let entry = grouped.entry(id).or_insert_with(|| ForeignKeyInfo {
                from: Vec::new(),
                to: Vec::new(),
                table: target_table.clone(),
            });
            while entry.from.len() <= seq as usize {
                entry.from.push(String::new());
                entry.to.push(String::new());
            }
            entry.from[seq as usize] = from;
            entry.to[seq as usize] = to;
            entry.table = target_table;
        }
        Ok(grouped.into_values().collect())
    }
}

// `connection` объявлен в `store.rs` как `pub(crate) fn connection(&self) -> &Connection`,
// поэтому доступ из этого модуля (того же крейта) работает напрямую.
#[allow(dead_code)]
fn _ensure_connection_visible(_c: &Connection) {}

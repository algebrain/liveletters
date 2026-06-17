use rusqlite::params;

use crate::{OutboxDelivery, OutboxRecord, Store, StoreError};

impl Store {
    pub fn save_outbox_record(&self, record: &OutboxRecord) -> Result<(), StoreError> {
        self.connection().execute(
            r#"
            INSERT OR REPLACE INTO outbox
                (event_id, event_type, author_email, resource_email, delivery_json,
                 message_body, message_id, subject, human_readable_body)
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                record.event_id,
                record.event_type,
                record.author_email,
                record.resource_email,
                encode_delivery(&record.delivery),
                record.message_body,
                record.message_id,
                record.subject,
                record.human_readable_body,
            ],
        )?;

        Ok(())
    }

    pub fn list_outbox_records(&self) -> Result<Vec<OutboxRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            r#"
            SELECT event_id, event_type, author_email, resource_email, delivery_json,
                   message_body, message_id, subject, human_readable_body
            FROM outbox
            ORDER BY rowid ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
            ))
        })?;

        let mut records = Vec::new();
        for row in rows {
            let (
                event_id,
                event_type,
                author_email,
                resource_email,
                delivery_json,
                message_body,
                message_id,
                subject,
                human_readable_body,
            ) = row?;
            let delivery = decode_delivery(&delivery_json)?;
            records.push(OutboxRecord {
                event_id,
                event_type,
                author_email,
                resource_email,
                delivery,
                message_body,
                message_id,
                subject,
                human_readable_body,
            });
        }

        Ok(records)
    }

    pub fn find_outbox_by_message_id(
        &self,
        message_id: &str,
    ) -> Result<Option<OutboxRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            r#"
            SELECT event_id, event_type, author_email, resource_email, delivery_json,
                   message_body, message_id, subject, human_readable_body
            FROM outbox
            WHERE message_id = ?1
            "#,
        )?;

        let mut rows = stmt.query(params![message_id])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        let event_id: String = row.get(0)?;
        let event_type: String = row.get(1)?;
        let author_email: String = row.get(2)?;
        let resource_email: Option<String> = row.get(3)?;
        let delivery_json: String = row.get(4)?;
        let message_body: String = row.get(5)?;
        let message_id: Option<String> = row.get(6)?;
        let subject: Option<String> = row.get(7)?;
        let human_readable_body: Option<String> = row.get(8)?;
        let delivery = decode_delivery(&delivery_json)?;
        Ok(Some(OutboxRecord {
            event_id,
            event_type,
            author_email,
            resource_email,
            delivery,
            message_body,
            message_id,
            subject,
            human_readable_body,
        }))
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

pub fn encode_delivery(delivery: &OutboxDelivery) -> String {
    match delivery {
        OutboxDelivery::Direct(addrs) => {
            let parts: Vec<String> = addrs
                .iter()
                .map(|a| format!("\"{}\"", json_escape(a)))
                .collect();
            format!(
                "{{\"kind\":\"direct\",\"addresses\":[{}]}}",
                parts.join(",")
            )
        }
        OutboxDelivery::ResourceSubscribers => "{\"kind\":\"resource_subscribers\"}".to_owned(),
    }
}

pub fn decode_delivery(raw: &str) -> Result<OutboxDelivery, StoreError> {
    if raw.contains("\"kind\":\"resource_subscribers\"") {
        return Ok(OutboxDelivery::ResourceSubscribers);
    }
    if raw.contains("\"kind\":\"direct\"") {
        return Ok(OutboxDelivery::Direct(parse_direct_addresses(raw)?));
    }
    Err(StoreError::InvalidColumn(format!(
        "delivery_json: неизвестный формат: {raw}"
    )))
}

fn parse_direct_addresses(raw: &str) -> Result<Vec<String>, StoreError> {
    let start = raw.find("\"addresses\":").ok_or_else(|| {
        StoreError::InvalidColumn("delivery_json: addresses не найден".to_owned())
    })?;
    let after = &raw[start..];
    let open = after
        .find('[')
        .ok_or_else(|| StoreError::InvalidColumn("delivery_json: addresses без [".to_owned()))?;
    let close_rel = after[open..]
        .find(']')
        .ok_or_else(|| StoreError::InvalidColumn("delivery_json: addresses без ]".to_owned()))?;
    let inner = &after[open + 1..open + close_rel];

    let mut addrs = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escape = false;
    for c in inner.chars() {
        if escape {
            match c {
                '"' => current.push('"'),
                '\\' => current.push('\\'),
                'n' => current.push('\n'),
                'r' => current.push('\r'),
                't' => current.push('\t'),
                'b' => current.push('\u{0008}'),
                'f' => current.push('\u{000C}'),
                '/' => current.push('/'),
                other => current.push(other),
            }
            escape = false;
            continue;
        }
        match c {
            '\\' if in_string => escape = true,
            '"' => {
                if in_string {
                    in_string = false;
                    addrs.push(std::mem::take(&mut current));
                } else {
                    in_string = true;
                }
            }
            ',' if !in_string => {}
            c if in_string => current.push(c),
            _ => {}
        }
    }
    Ok(addrs)
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_resource_subscribers_round_trip() {
        let value = OutboxDelivery::ResourceSubscribers;
        let encoded = encode_delivery(&value);
        let decoded = decode_delivery(&encoded).expect("decode");
        assert_eq!(decoded, value);
    }

    #[test]
    fn encode_decode_direct_round_trip() {
        let value = OutboxDelivery::Direct(vec!["a@example.org".into(), "b@example.org".into()]);
        let encoded = encode_delivery(&value);
        let decoded = decode_delivery(&encoded).expect("decode");
        assert_eq!(decoded, value);
    }

    #[test]
    fn encode_direct_escapes_specials() {
        let value = OutboxDelivery::Direct(vec!["a\"b\\c".into()]);
        let encoded = encode_delivery(&value);
        assert!(encoded.contains("\\\""));
        assert!(encoded.contains("\\\\"));
        let decoded = decode_delivery(&encoded).expect("decode");
        assert_eq!(decoded, value);
    }

    #[test]
    fn decode_unknown_kind_is_error() {
        let err = decode_delivery("{\"kind\":\"other\"}").unwrap_err();
        assert!(matches!(err, StoreError::InvalidColumn(_)));
    }
}

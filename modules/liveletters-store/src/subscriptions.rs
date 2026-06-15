use rusqlite::params;

use crate::{PendingSubscriptionRecord, Store, StoreError, SubscriptionRecord};

impl Store {
    pub fn save_subscription(&self, record: &SubscriptionRecord) -> Result<(), StoreError> {
        self.connection().execute(
            r#"
            INSERT OR REPLACE INTO subscriptions
                (resource_address, subscriber_delivery_address)
            VALUES
                (?1, ?2)
            "#,
            params![record.resource_address, record.subscriber_delivery_address],
        )?;

        Ok(())
    }

    pub fn delete_subscription(
        &self,
        resource_address: &str,
        subscriber_delivery_address: &str,
    ) -> Result<bool, StoreError> {
        let changed = self.connection().execute(
            r#"
            DELETE FROM subscriptions
            WHERE resource_address = ?1 AND subscriber_delivery_address = ?2
            "#,
            params![resource_address, subscriber_delivery_address],
        )?;

        Ok(changed > 0)
    }

    pub fn list_subscriptions_for_resource(
        &self,
        resource_address: &str,
    ) -> Result<Vec<SubscriptionRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            r#"
            SELECT resource_address, subscriber_delivery_address
            FROM subscriptions
            WHERE resource_address = ?1
            ORDER BY subscriber_delivery_address ASC
            "#,
        )?;

        let rows = stmt.query_map(params![resource_address], |row| {
            Ok(SubscriptionRecord {
                resource_address: row.get(0)?,
                subscriber_delivery_address: row.get(1)?,
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }

        Ok(records)
    }

    pub fn list_subscriptions_for_subscriber(
        &self,
        subscriber_delivery_address: &str,
    ) -> Result<Vec<SubscriptionRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            r#"
            SELECT resource_address, subscriber_delivery_address
            FROM subscriptions
            WHERE subscriber_delivery_address = ?1
            ORDER BY resource_address ASC
            "#,
        )?;

        let rows = stmt.query_map(params![subscriber_delivery_address], |row| {
            Ok(SubscriptionRecord {
                resource_address: row.get(0)?,
                subscriber_delivery_address: row.get(1)?,
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }

        Ok(records)
    }

    pub fn save_pending_subscription(
        &self,
        profile_id: &str,
        resource_address: &str,
        requested_at: u64,
    ) -> Result<(), StoreError> {
        // UPSERT: при существующей записи сохраняем `requested_at` (не перезаписываем),
        // обновляем `last_attempt_at` до `requested_at` (первая попытка).
        self.connection().execute(
            r#"
            INSERT INTO pending_subscriptions
                (profile_id, resource_address, requested_at, last_attempt_at)
            VALUES
                (?1, ?2, ?3, ?3)
            ON CONFLICT (profile_id, resource_address) DO UPDATE
            SET last_attempt_at = excluded.last_attempt_at
            "#,
            params![profile_id, resource_address, requested_at as i64],
        )?;
        Ok(())
    }

    pub fn update_pending_last_attempt(
        &self,
        profile_id: &str,
        resource_address: &str,
        last_attempt_at: u64,
    ) -> Result<(), StoreError> {
        self.connection().execute(
            "UPDATE pending_subscriptions SET last_attempt_at = ?1 WHERE profile_id = ?2 AND resource_address = ?3",
            params![last_attempt_at as i64, profile_id, resource_address],
        )?;
        Ok(())
    }

    pub fn remove_pending_subscription(
        &self,
        profile_id: &str,
        resource_address: &str,
    ) -> Result<(), StoreError> {
        self.connection().execute(
            "DELETE FROM pending_subscriptions WHERE profile_id = ?1 AND resource_address = ?2",
            params![profile_id, resource_address],
        )?;
        Ok(())
    }

    pub fn list_pending_subscriptions(
        &self,
        profile_id: &str,
    ) -> Result<Vec<PendingSubscriptionRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            "SELECT profile_id, resource_address, requested_at, last_attempt_at
             FROM pending_subscriptions WHERE profile_id = ?1
             ORDER BY requested_at",
        )?;
        let rows = stmt.query_map(params![profile_id], |r| {
            Ok(PendingSubscriptionRecord {
                profile_id: r.get(0)?,
                resource_address: r.get(1)?,
                requested_at: r.get::<_, i64>(2)? as u64,
                last_attempt_at: r.get::<_, i64>(3)? as u64,
            })
        })?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn find_pending_subscription(
        &self,
        profile_id: &str,
        resource_address: &str,
    ) -> Result<Option<PendingSubscriptionRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            "SELECT profile_id, resource_address, requested_at, last_attempt_at
             FROM pending_subscriptions WHERE profile_id = ?1 AND resource_address = ?2",
        )?;
        let mut rows = stmt.query(params![profile_id, resource_address])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        Ok(Some(PendingSubscriptionRecord {
            profile_id: row.get(0)?,
            resource_address: row.get(1)?,
            requested_at: row.get::<_, i64>(2)? as u64,
            last_attempt_at: row.get::<_, i64>(3)? as u64,
        }))
    }
}

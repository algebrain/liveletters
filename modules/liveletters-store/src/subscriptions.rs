use rusqlite::params;

use crate::{PendingSubscriptionRecord, Store, StoreError, SubscriptionRecord};

impl Store {
    pub fn save_subscription(&self, record: &SubscriptionRecord) -> Result<(), StoreError> {
        self.connection().execute(
            r#"
            INSERT OR REPLACE INTO subscriptions
                (resource_email, subscriber_email)
            VALUES
                (?1, ?2)
            "#,
            params![record.resource_email, record.subscriber_email],
        )?;

        Ok(())
    }

    pub fn delete_subscription(
        &self,
        resource_email: &str,
        subscriber_email: &str,
    ) -> Result<bool, StoreError> {
        let changed = self.connection().execute(
            r#"
            DELETE FROM subscriptions
            WHERE resource_email = ?1 AND subscriber_email = ?2
            "#,
            params![resource_email, subscriber_email],
        )?;

        Ok(changed > 0)
    }

    pub fn list_subscriptions_for_resource(
        &self,
        resource_email: &str,
    ) -> Result<Vec<SubscriptionRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            r#"
            SELECT resource_email, subscriber_email
            FROM subscriptions
            WHERE resource_email = ?1
            ORDER BY subscriber_email ASC
            "#,
        )?;

        let rows = stmt.query_map(params![resource_email], |row| {
            Ok(SubscriptionRecord {
                resource_email: row.get(0)?,
                subscriber_email: row.get(1)?,
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
        subscriber_email: &str,
    ) -> Result<Vec<SubscriptionRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            r#"
            SELECT resource_email, subscriber_email
            FROM subscriptions
            WHERE subscriber_email = ?1
            ORDER BY resource_email ASC
            "#,
        )?;

        let rows = stmt.query_map(params![subscriber_email], |row| {
            Ok(SubscriptionRecord {
                resource_email: row.get(0)?,
                subscriber_email: row.get(1)?,
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
        resource_email: &str,
        requested_at: u64,
    ) -> Result<(), StoreError> {
        // UPSERT: при существующей записи сохраняем `requested_at` (не перезаписываем),
        // обновляем `last_attempt_at` до `requested_at` (первая попытка).
        self.connection().execute(
            r#"
            INSERT INTO pending_subscriptions
                (profile_id, resource_email, requested_at, last_attempt_at)
            VALUES
                (?1, ?2, ?3, ?3)
            ON CONFLICT (profile_id, resource_email) DO UPDATE
            SET last_attempt_at = excluded.last_attempt_at
            "#,
            params![profile_id, resource_email, requested_at as i64],
        )?;
        Ok(())
    }

    pub fn update_pending_last_attempt(
        &self,
        profile_id: &str,
        resource_email: &str,
        last_attempt_at: u64,
    ) -> Result<(), StoreError> {
        self.connection().execute(
            "UPDATE pending_subscriptions SET last_attempt_at = ?1 WHERE profile_id = ?2 AND resource_email = ?3",
            params![last_attempt_at as i64, profile_id, resource_email],
        )?;
        Ok(())
    }

    pub fn remove_pending_subscription(
        &self,
        profile_id: &str,
        resource_email: &str,
    ) -> Result<(), StoreError> {
        self.connection().execute(
            "DELETE FROM pending_subscriptions WHERE profile_id = ?1 AND resource_email = ?2",
            params![profile_id, resource_email],
        )?;
        Ok(())
    }

    pub fn list_pending_subscriptions(
        &self,
        profile_id: &str,
    ) -> Result<Vec<PendingSubscriptionRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            "SELECT profile_id, resource_email, requested_at, last_attempt_at
             FROM pending_subscriptions WHERE profile_id = ?1
             ORDER BY requested_at",
        )?;
        let rows = stmt.query_map(params![profile_id], |r| {
            Ok(PendingSubscriptionRecord {
                profile_id: r.get(0)?,
                resource_email: r.get(1)?,
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
        resource_email: &str,
    ) -> Result<Option<PendingSubscriptionRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            "SELECT profile_id, resource_email, requested_at, last_attempt_at
             FROM pending_subscriptions WHERE profile_id = ?1 AND resource_email = ?2",
        )?;
        let mut rows = stmt.query(params![profile_id, resource_email])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        Ok(Some(PendingSubscriptionRecord {
            profile_id: row.get(0)?,
            resource_email: row.get(1)?,
            requested_at: row.get::<_, i64>(2)? as u64,
            last_attempt_at: row.get::<_, i64>(3)? as u64,
        }))
    }
}

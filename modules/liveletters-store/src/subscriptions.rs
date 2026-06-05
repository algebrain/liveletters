use rusqlite::params;

use crate::{Store, StoreError, SubscriptionRecord};

impl Store {
    pub fn save_subscription(&self, record: &SubscriptionRecord) -> Result<(), StoreError> {
        self.connection().execute(
            r#"
            INSERT OR REPLACE INTO subscriptions
                (resource_address, subscriber_account_id, subscriber_delivery_address)
            VALUES
                (?1, ?2, ?3)
            "#,
            params![
                record.resource_address,
                record.subscriber_account_id,
                record.subscriber_delivery_address,
            ],
        )?;

        Ok(())
    }

    pub fn delete_subscription(
        &self,
        resource_address: &str,
        subscriber_account_id: &str,
    ) -> Result<bool, StoreError> {
        let changed = self.connection().execute(
            r#"
            DELETE FROM subscriptions
            WHERE resource_address = ?1 AND subscriber_account_id = ?2
            "#,
            params![resource_address, subscriber_account_id],
        )?;

        Ok(changed > 0)
    }

    pub fn list_subscriptions_for_resource(
        &self,
        resource_address: &str,
    ) -> Result<Vec<SubscriptionRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            r#"
            SELECT resource_address, subscriber_account_id, subscriber_delivery_address
            FROM subscriptions
            WHERE resource_address = ?1
            ORDER BY subscriber_account_id ASC
            "#,
        )?;

        let rows = stmt.query_map(params![resource_address], |row| {
            Ok(SubscriptionRecord {
                resource_address: row.get(0)?,
                subscriber_account_id: row.get(1)?,
                subscriber_delivery_address: row.get(2)?,
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
        subscriber_account_id: &str,
    ) -> Result<Vec<SubscriptionRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            r#"
            SELECT resource_address, subscriber_account_id, subscriber_delivery_address
            FROM subscriptions
            WHERE subscriber_account_id = ?1
            ORDER BY resource_address ASC
            "#,
        )?;

        let rows = stmt.query_map(params![subscriber_account_id], |row| {
            Ok(SubscriptionRecord {
                resource_address: row.get(0)?,
                subscriber_account_id: row.get(1)?,
                subscriber_delivery_address: row.get(2)?,
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }

        Ok(records)
    }
}

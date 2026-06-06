use rusqlite::params;

use crate::{Store, StoreError, SubscriptionRecord};

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
}

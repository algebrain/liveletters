use rusqlite::{OptionalExtension, params};

use crate::{FriendOfRecord, FriendRecord, PendingFriendRecord, Store, StoreError};

impl Store {
    pub fn save_friend(
        &self,
        owner_resource_email: &str,
        friend_email: &str,
    ) -> Result<(), StoreError> {
        self.connection().execute(
            r#"
            INSERT OR IGNORE INTO friends
                (owner_resource_email, friend_email)
            VALUES
                (?1, ?2)
            "#,
            params![owner_resource_email, friend_email],
        )?;
        Ok(())
    }

    pub fn delete_friend(
        &self,
        owner_resource_email: &str,
        friend_email: &str,
    ) -> Result<bool, StoreError> {
        let changed = self.connection().execute(
            r#"
            DELETE FROM friends
            WHERE owner_resource_email = ?1 AND friend_email = ?2
            "#,
            params![owner_resource_email, friend_email],
        )?;
        Ok(changed > 0)
    }

    pub fn list_friends_for_resource(
        &self,
        owner_resource_email: &str,
    ) -> Result<Vec<FriendRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            r#"
            SELECT owner_resource_email, friend_email
            FROM friends
            WHERE owner_resource_email = ?1
            ORDER BY friend_email ASC
            "#,
        )?;
        let rows = stmt.query_map(params![owner_resource_email], |row| {
            Ok(FriendRecord {
                owner_resource_email: row.get(0)?,
                friend_email: row.get(1)?,
            })
        })?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn is_friend(
        &self,
        owner_resource_email: &str,
        friend_email: &str,
    ) -> Result<bool, StoreError> {
        let exists: Option<i64> = self
            .connection()
            .query_row(
                r#"
                SELECT 1 FROM friends
                WHERE owner_resource_email = ?1 AND friend_email = ?2
                "#,
                params![owner_resource_email, friend_email],
                |row| row.get(0),
            )
            .optional()?;
        Ok(exists.is_some())
    }

    pub fn save_pending_friend(
        &self,
        profile_id: &str,
        owner_resource_email: &str,
        friend_email: &str,
        subscribed_resource_email: &str,
        requested_at: u64,
    ) -> Result<(), StoreError> {
        self.connection().execute(
            r#"
            INSERT INTO pending_friends
                (profile_id, owner_resource_email, friend_email,
                 subscribed_resource_email, requested_at, last_attempt_at)
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?5)
            ON CONFLICT (profile_id, owner_resource_email, friend_email) DO UPDATE
            SET last_attempt_at = excluded.last_attempt_at,
                subscribed_resource_email = excluded.subscribed_resource_email
            "#,
            params![
                profile_id,
                owner_resource_email,
                friend_email,
                subscribed_resource_email,
                requested_at as i64
            ],
        )?;
        Ok(())
    }

    pub fn remove_pending_friend(
        &self,
        profile_id: &str,
        owner_resource_email: &str,
        friend_email: &str,
    ) -> Result<(), StoreError> {
        self.connection().execute(
            r#"
            DELETE FROM pending_friends
            WHERE profile_id = ?1 AND owner_resource_email = ?2 AND friend_email = ?3
            "#,
            params![profile_id, owner_resource_email, friend_email],
        )?;
        Ok(())
    }

    pub fn get_pending_friend(
        &self,
        profile_id: &str,
        owner_resource_email: &str,
        friend_email: &str,
    ) -> Result<Option<PendingFriendRecord>, StoreError> {
        self.connection()
            .query_row(
                r#"
                SELECT profile_id, owner_resource_email, friend_email,
                       subscribed_resource_email, requested_at, last_attempt_at
                FROM pending_friends
                WHERE profile_id = ?1 AND owner_resource_email = ?2 AND friend_email = ?3
                "#,
                params![profile_id, owner_resource_email, friend_email],
                pending_friend_from_row,
            )
            .optional()
            .map_err(StoreError::from)
    }

    pub fn find_pending_friend_by_subscribed_resource(
        &self,
        profile_id: &str,
        subscribed_resource_email: &str,
    ) -> Result<Option<PendingFriendRecord>, StoreError> {
        self.connection()
            .query_row(
                r#"
                SELECT profile_id, owner_resource_email, friend_email,
                       subscribed_resource_email, requested_at, last_attempt_at
                FROM pending_friends
                WHERE profile_id = ?1 AND subscribed_resource_email = ?2
                ORDER BY requested_at ASC
                LIMIT 1
                "#,
                params![profile_id, subscribed_resource_email],
                pending_friend_from_row,
            )
            .optional()
            .map_err(StoreError::from)
    }

    pub fn list_pending_friends(
        &self,
        profile_id: &str,
    ) -> Result<Vec<PendingFriendRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            r#"
            SELECT profile_id, owner_resource_email, friend_email,
                   subscribed_resource_email, requested_at, last_attempt_at
            FROM pending_friends
            WHERE profile_id = ?1
            ORDER BY requested_at ASC
            "#,
        )?;
        let rows = stmt.query_map(params![profile_id], pending_friend_from_row)?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn save_friend_of(&self, profile_id: &str, resource_email: &str) -> Result<(), StoreError> {
        self.connection().execute(
            r#"
            INSERT OR IGNORE INTO friend_of
                (profile_id, resource_email)
            VALUES
                (?1, ?2)
            "#,
            params![profile_id, resource_email],
        )?;
        Ok(())
    }

    pub fn list_friend_of(&self, profile_id: &str) -> Result<Vec<FriendOfRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            r#"
            SELECT profile_id, resource_email
            FROM friend_of
            WHERE profile_id = ?1
            ORDER BY resource_email ASC
            "#,
        )?;
        let rows = stmt.query_map(params![profile_id], |row| {
            Ok(FriendOfRecord {
                profile_id: row.get(0)?,
                resource_email: row.get(1)?,
            })
        })?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }
}

fn pending_friend_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PendingFriendRecord> {
    Ok(PendingFriendRecord {
        profile_id: row.get(0)?,
        owner_resource_email: row.get(1)?,
        friend_email: row.get(2)?,
        subscribed_resource_email: row.get(3)?,
        requested_at: row.get::<_, i64>(4)? as u64,
        last_attempt_at: row.get::<_, i64>(5)? as u64,
    })
}

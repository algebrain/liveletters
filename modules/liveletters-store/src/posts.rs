use rusqlite::params;

use crate::{PostRecord, Store, StoreError};

impl Store {
    pub fn save_post_record(&self, post: &PostRecord) -> Result<(), StoreError> {
        self.connection().execute(
            r#"
            INSERT OR REPLACE INTO posts
                (post_id, resource_id, author_id, created_at, body, visibility, hidden)
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                post.post_id,
                post.resource_id,
                post.author_id,
                post.created_at as i64,
                post.body,
                post.visibility,
                post.hidden as i64,
            ],
        )?;

        Ok(())
    }

    pub fn list_posts(&self) -> Result<Vec<PostRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            r#"
            SELECT post_id, resource_id, author_id, created_at, body, visibility, hidden
            FROM posts
            ORDER BY created_at ASC, post_id ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(PostRecord {
                post_id: row.get(0)?,
                resource_id: row.get(1)?,
                author_id: row.get(2)?,
                created_at: row.get::<_, i64>(3)? as u64,
                body: row.get(4)?,
                visibility: row.get(5)?,
                hidden: row.get::<_, i64>(6)? != 0,
            })
        })?;

        let mut posts = Vec::new();
        for row in rows {
            posts.push(row?);
        }

        Ok(posts)
    }

    pub fn count_posts(&self) -> Result<u64, StoreError> {
        let n: i64 = self
            .connection()
            .query_row("SELECT COUNT(*) FROM posts", [], |row| row.get(0))?;
        Ok(n as u64)
    }

    pub fn newest_post_created_at(&self) -> Result<Option<u64>, StoreError> {
        let ts: Option<i64> =
            self.connection()
                .query_row("SELECT MAX(created_at) FROM posts", [], |row| row.get(0))?;
        Ok(ts.map(|t| t as u64))
    }

    pub fn get_post_record(&self, post_id: &str) -> Result<Option<PostRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            r#"
            SELECT post_id, resource_id, author_id, created_at, body, visibility, hidden
            FROM posts
            WHERE post_id = ?1
            "#,
        )?;

        let mut rows = stmt.query([post_id])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };

        Ok(Some(PostRecord {
            post_id: row.get(0)?,
            resource_id: row.get(1)?,
            author_id: row.get(2)?,
            created_at: row.get::<_, i64>(3)? as u64,
            body: row.get(4)?,
            visibility: row.get(5)?,
            hidden: row.get::<_, i64>(6)? != 0,
        }))
    }
}

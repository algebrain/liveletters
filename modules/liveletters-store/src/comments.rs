use crate::{CommentRecord, Store, StoreError};

impl Store {
    pub fn save_comment_record(&self, comment: &CommentRecord) -> Result<(), StoreError> {
        self.connection().execute(
            r#"
            INSERT OR REPLACE INTO comments
                (comment_id, post_id, parent_comment_id, author_id, created_at, body, visibility, hidden)
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            rusqlite::params![
                comment.comment_id,
                comment.post_id,
                comment.parent_comment_id,
                comment.author_id,
                comment.created_at as i64,
                comment.body,
                comment.visibility,
                comment.hidden as i64,
            ],
        )?;

        Ok(())
    }

    pub fn list_comments_for_post(&self, post_id: &str) -> Result<Vec<CommentRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            r#"
            SELECT comment_id, post_id, parent_comment_id, author_id, created_at, body, visibility, hidden
            FROM comments
            WHERE post_id = ?1
            ORDER BY created_at ASC, comment_id ASC
            "#,
        )?;

        let rows = stmt.query_map([post_id], |row| {
            Ok(CommentRecord {
                comment_id: row.get(0)?,
                post_id: row.get(1)?,
                parent_comment_id: row.get(2)?,
                author_id: row.get(3)?,
                created_at: row.get::<_, i64>(4)? as u64,
                body: row.get(5)?,
                visibility: row.get(6)?,
                hidden: row.get::<_, i64>(7)? != 0,
            })
        })?;

        let mut comments = Vec::new();
        for row in rows {
            comments.push(row?);
        }

        Ok(comments)
    }

    pub fn count_comments(&self) -> Result<u64, StoreError> {
        let n: i64 = self
            .connection()
            .query_row("SELECT COUNT(*) FROM comments", [], |row| row.get(0))?;
        Ok(n as u64)
    }

    pub fn get_comment_record(
        &self,
        comment_id: &str,
    ) -> Result<Option<CommentRecord>, StoreError> {
        let mut stmt = self.connection().prepare(
            r#"
            SELECT comment_id, post_id, parent_comment_id, author_id, created_at, body, visibility, hidden
            FROM comments
            WHERE comment_id = ?1
            "#,
        )?;

        let mut rows = stmt.query([comment_id])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };

        Ok(Some(CommentRecord {
            comment_id: row.get(0)?,
            post_id: row.get(1)?,
            parent_comment_id: row.get(2)?,
            author_id: row.get(3)?,
            created_at: row.get::<_, i64>(4)? as u64,
            body: row.get(5)?,
            visibility: row.get(6)?,
            hidden: row.get::<_, i64>(7)? != 0,
        }))
    }
}

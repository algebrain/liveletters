use std::{fs, path::Path};

use rusqlite::{params, Connection};

use crate::{CommentRecord, PostRecord, StoreError, StorePaths};

pub struct Store {
    connection: Connection,
}

impl Store {
    pub fn open_default() -> Result<Self, StoreError> {
        let paths = StorePaths::from_environment()?;
        Self::open_at(paths.database_path())
    }

    pub fn open_in_memory() -> Result<Self, StoreError> {
        let connection = Connection::open_in_memory()?;
        let store = Self { connection };
        store.initialize_schema()?;
        Ok(store)
    }

    pub fn open_at(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let connection = Connection::open(path)?;
        let store = Self { connection };
        store.initialize_schema()?;
        Ok(store)
    }

    fn initialize_schema(&self) -> Result<(), StoreError> {
        self.connection.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS posts (
                post_id TEXT PRIMARY KEY,
                resource_id TEXT NOT NULL,
                author_id TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                body TEXT NOT NULL,
                visibility TEXT NOT NULL,
                hidden INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS comments (
                comment_id TEXT PRIMARY KEY,
                post_id TEXT NOT NULL,
                parent_comment_id TEXT,
                author_id TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                body TEXT NOT NULL,
                visibility TEXT NOT NULL,
                hidden INTEGER NOT NULL
            );
            "#,
        )?;

        Ok(())
    }

    pub fn save_post_record(&self, post: &PostRecord) -> Result<(), StoreError> {
        self.connection.execute(
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

    pub fn save_comment_record(&self, comment: &CommentRecord) -> Result<(), StoreError> {
        self.connection.execute(
            r#"
            INSERT OR REPLACE INTO comments
                (comment_id, post_id, parent_comment_id, author_id, created_at, body, visibility, hidden)
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
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

    pub fn list_posts(&self) -> Result<Vec<PostRecord>, StoreError> {
        let mut stmt = self.connection.prepare(
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

    pub fn list_comments_for_post(&self, post_id: &str) -> Result<Vec<CommentRecord>, StoreError> {
        let mut stmt = self.connection.prepare(
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
}

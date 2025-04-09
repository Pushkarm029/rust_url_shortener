use async_trait::async_trait;
use sqlx::{Pool, Sqlite, SqlitePool, sqlite::SqlitePoolOptions};

use crate::errors::{AppError, Result};
use crate::models::{UnixNanos, Url};
use crate::storage;
use sqlx::Row;
use std::fs::File;
use std::path::Path;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

// SQLite implementation of the Storage trait
pub struct SqliteStorage {
    pool: Pool<Sqlite>,
}

impl SqliteStorage {
    // Create a new SQLite storage instance
    pub async fn new(database_url: &str) -> Result<Self> {
        // Check if database file exists, if not create an empty one
        if database_url.starts_with("sqlite:") {
            let path_str = database_url.trim_start_matches("sqlite:");
            let db_path = Path::new(path_str);

            if !db_path.exists() {
                tracing::info!(
                    "First run detected, creating new database file at {}",
                    path_str
                );
                File::create(db_path).map_err(|e| {
                    AppError::InternalError(format!("Failed to create database file: {}", e))
                })?;
            }
        }

        // Create a connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .map_err(AppError::DatabaseError)?;

        // Run migrations to create the table if it doesn't exist
        Self::run_migrations(&pool).await?;

        Ok(Self { pool })
    }

    // Run database migrations
    async fn run_migrations(pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS urls (
                short_id TEXT PRIMARY KEY,
                original_url TEXT NOT NULL,
                visit_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                last_accessed TEXT
            )
            "#,
        )
        .execute(pool)
        .await
        .map_err(AppError::DatabaseError)?;

        Ok(())
    }
}

#[async_trait]
impl storage::Storage for SqliteStorage {
    async fn create_url(&self, short_id: &str, original_url: &str) -> Result<Url> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        sqlx::query(
            r#"
            INSERT INTO urls (short_id, original_url, visit_count, created_at, last_accessed)
            VALUES (?, ?, 0, ?, NULL)
            "#,
        )
        .bind(short_id)
        .bind(original_url)
        .bind(now.to_string())
        .execute(&self.pool)
        .await
        .map_err(AppError::DatabaseError)?;

        Ok(Url {
            short_id: short_id.to_string(),
            original_url: original_url.to_string(),
            visit_count: 0,
            created_at: now.into(),
            last_accessed: None,
        })
    }

    async fn get_url(&self, short_id: &str) -> Result<Url> {
        let row = sqlx::query(
            r#"
            SELECT 
                short_id, 
                original_url, 
                visit_count, 
                created_at, 
                last_accessed
            FROM urls 
            WHERE short_id = ?
            "#,
        )
        .bind(short_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                AppError::NotFound(format!("URL with ID {} not found", short_id))
            }
            _ => AppError::DatabaseError(e),
        })?;

        let created_at_str: String = row.get("created_at");
        let last_accessed_str: Option<String> = row.get("last_accessed");

        Ok(Url {
            short_id: row.get("short_id"),
            original_url: row.get("original_url"),
            visit_count: row.get("visit_count"),
            created_at: UnixNanos::from_str(&created_at_str)
                .map_err(|_| AppError::InternalError("Invalid timestamp format".to_string()))?,
            last_accessed: last_accessed_str
                .map(|s| UnixNanos::from_str(&s))
                .transpose()
                .map_err(|_| AppError::InternalError("Invalid timestamp format".to_string()))?,
        })
    }

    async fn increment_visit(&self, short_id: &str) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        let rows_affected = sqlx::query(
            r#"
            UPDATE urls
            SET visit_count = visit_count + 1, last_accessed = ?
            WHERE short_id = ?
            "#,
        )
        .bind(now.to_string())
        .bind(short_id)
        .execute(&self.pool)
        .await
        .map_err(AppError::DatabaseError)?
        .rows_affected();

        if rows_affected == 0 {
            return Err(AppError::NotFound(format!(
                "URL with ID {} not found",
                short_id
            )));
        }

        Ok(())
    }

    async fn url_exists(&self, short_id: &str) -> Result<bool> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM urls 
            WHERE short_id = ?
            "#,
        )
        .bind(short_id)
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::DatabaseError)?;

        let count: i64 = row.get("count");

        Ok(count > 0)
    }

    async fn list_urls(&self, limit: usize, offset: usize) -> Result<Vec<Url>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                short_id, 
                original_url, 
                visit_count, 
                created_at, 
                last_accessed
            FROM urls 
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::DatabaseError)?;

        let mut urls = Vec::with_capacity(rows.len());

        for row in rows {
            let created_at_str: String = row.get("created_at");
            let last_accessed_str: Option<String> = row.get("last_accessed");

            urls.push(Url {
                short_id: row.get("short_id"),
                original_url: row.get("original_url"),
                visit_count: row.get("visit_count"),
                created_at: UnixNanos::from_str(&created_at_str)
                    .map_err(|_| AppError::InternalError("Invalid timestamp format".to_string()))?,
                last_accessed: last_accessed_str
                    .map(|s| UnixNanos::from_str(&s))
                    .transpose()
                    .map_err(|_| AppError::InternalError("Invalid timestamp format".to_string()))?,
            });
        }

        Ok(urls)
    }
}

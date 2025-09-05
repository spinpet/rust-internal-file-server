use crate::error::{Result, ServerError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::{query, Row};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: String,
    pub original_name: String,
    pub stored_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
    pub upload_time: DateTime<Utc>,
    pub is_video: bool,
    pub thumbnail_path: Option<String>,
    pub video_duration: Option<i32>,
    pub video_resolution: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FileManager {
    pool: SqlitePool,
    storage_path: PathBuf,
}

impl FileManager {
    pub async fn new(database_url: &str, storage_path: PathBuf) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(20)
            .connect(database_url)
            .await
            .map_err(|e| ServerError::Database(e))?;

        let manager = Self { pool, storage_path };
        manager.init().await?;
        Ok(manager)
    }

    pub async fn init(&self) -> Result<()> {
        std::fs::create_dir_all(&self.storage_path)
            .map_err(|e| ServerError::Io(e))?;

        let create_files_table = r#"
            CREATE TABLE IF NOT EXISTS files (
                id TEXT PRIMARY KEY,
                original_name TEXT NOT NULL,
                stored_name TEXT NOT NULL,
                file_path TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                mime_type TEXT NOT NULL,
                upload_time TEXT NOT NULL,
                is_video BOOLEAN NOT NULL DEFAULT FALSE,
                thumbnail_path TEXT,
                video_duration INTEGER,
                video_resolution TEXT
            )
        "#;

        query(create_files_table)
            .execute(&self.pool)
            .await
            .map_err(|e| ServerError::Database(e))?;

        let create_index = r#"
            CREATE INDEX IF NOT EXISTS idx_upload_time ON files(upload_time DESC);
            CREATE INDEX IF NOT EXISTS idx_is_video ON files(is_video);
            CREATE INDEX IF NOT EXISTS idx_file_size ON files(file_size DESC);
        "#;

        query(create_index)
            .execute(&self.pool)
            .await
            .map_err(|e| ServerError::Database(e))?;

        Ok(())
    }

    pub async fn save_file_record(&self, record: &FileRecord) -> Result<()> {
        let sql = r#"
            INSERT INTO files (
                id, original_name, stored_name, file_path, file_size, mime_type, 
                upload_time, is_video, thumbnail_path, video_duration, video_resolution
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        query(sql)
            .bind(&record.id)
            .bind(&record.original_name)
            .bind(&record.stored_name)
            .bind(&record.file_path)
            .bind(record.file_size)
            .bind(&record.mime_type)
            .bind(record.upload_time.to_rfc3339())
            .bind(record.is_video)
            .bind(&record.thumbnail_path)
            .bind(record.video_duration)
            .bind(&record.video_resolution)
            .execute(&self.pool)
            .await
            .map_err(|e| ServerError::Database(e))?;

        Ok(())
    }

    pub async fn get_file_by_id(&self, file_id: &str) -> Result<Option<FileRecord>> {
        let sql = "SELECT * FROM files WHERE id = ?";
        
        let row = query(sql)
            .bind(file_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ServerError::Database(e))?;

        if let Some(row) = row {
            let upload_time_str: String = row.get("upload_time");
            let upload_time = DateTime::parse_from_rfc3339(&upload_time_str)
                .map_err(|e| ServerError::Internal(e.into()))?
                .with_timezone(&Utc);

            Ok(Some(FileRecord {
                id: row.get("id"),
                original_name: row.get("original_name"),
                stored_name: row.get("stored_name"),
                file_path: row.get("file_path"),
                file_size: row.get("file_size"),
                mime_type: row.get("mime_type"),
                upload_time,
                is_video: row.get("is_video"),
                thumbnail_path: row.get("thumbnail_path"),
                video_duration: row.get("video_duration"),
                video_resolution: row.get("video_resolution"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_files(&self, limit: Option<i32>, offset: Option<i32>) -> Result<Vec<FileRecord>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);
        
        let sql = "SELECT * FROM files ORDER BY upload_time DESC LIMIT ? OFFSET ?";
        
        let rows = query(sql)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ServerError::Database(e))?;

        let mut files = Vec::new();
        for row in rows {
            let upload_time_str: String = row.get("upload_time");
            let upload_time = DateTime::parse_from_rfc3339(&upload_time_str)
                .map_err(|e| ServerError::Internal(e.into()))?
                .with_timezone(&Utc);

            files.push(FileRecord {
                id: row.get("id"),
                original_name: row.get("original_name"),
                stored_name: row.get("stored_name"),
                file_path: row.get("file_path"),
                file_size: row.get("file_size"),
                mime_type: row.get("mime_type"),
                upload_time,
                is_video: row.get("is_video"),
                thumbnail_path: row.get("thumbnail_path"),
                video_duration: row.get("video_duration"),
                video_resolution: row.get("video_resolution"),
            });
        }

        Ok(files)
    }

    pub async fn delete_file(&self, file_id: &str) -> Result<bool> {
        if let Some(record) = self.get_file_by_id(file_id).await? {
            let file_path = Path::new(&record.file_path);
            if file_path.exists() {
                std::fs::remove_file(file_path)
                    .map_err(|e| ServerError::Io(e))?;
            }

            if let Some(thumbnail) = &record.thumbnail_path {
                let thumb_path = Path::new(thumbnail);
                if thumb_path.exists() {
                    let _ = std::fs::remove_file(thumb_path);
                }
            }

            let sql = "DELETE FROM files WHERE id = ?";
            query(sql)
                .bind(file_id)
                .execute(&self.pool)
                .await
                .map_err(|e| ServerError::Database(e))?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn get_file_stats(&self) -> Result<FileStats> {
        let sql = r#"
            SELECT 
                COUNT(*) as total_files,
                SUM(file_size) as total_size,
                COUNT(CASE WHEN is_video = 1 THEN 1 END) as video_count
            FROM files
        "#;

        let row = query(sql)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ServerError::Database(e))?;

        Ok(FileStats {
            total_files: row.get::<i64, _>("total_files") as u64,
            total_size: row.get::<Option<i64>, _>("total_size").unwrap_or(0) as u64,
            video_count: row.get::<i64, _>("video_count") as u64,
        })
    }

    pub fn generate_stored_name(&self, original_name: &str) -> String {
        let extension = Path::new(original_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        let uuid = Uuid::new_v4();
        if extension.is_empty() {
            uuid.to_string()
        } else {
            format!("{}.{}", uuid, extension)
        }
    }

    pub fn get_storage_path(&self) -> &Path {
        &self.storage_path
    }

    pub fn get_file_path(&self, stored_name: &str) -> PathBuf {
        self.storage_path.join(stored_name)
    }
}

#[derive(Debug, Serialize)]
pub struct FileStats {
    pub total_files: u64,
    pub total_size: u64,
    pub video_count: u64,
}
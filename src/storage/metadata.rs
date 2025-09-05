// 文件元数据结构
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: i64,
    pub filename: String,
    pub size: i64,
    pub mime_type: Option<String>,
    pub upload_time: DateTime<Utc>,
    pub file_path: String,
}
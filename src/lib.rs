pub mod config;
pub mod error;
pub mod server;
pub mod storage;
pub mod upload;
pub mod download;
pub mod video;
pub mod web;

pub use error::{Result, ServerError};

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::storage;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.server.address, "0.0.0.0");
        assert_eq!(config.server.port, 3000);
    }

    #[test]
    fn test_server_address() {
        let config = Config::default();
        assert_eq!(config.server_address(), "0.0.0.0:3000");
    }

    #[tokio::test]
    async fn test_storage_file_manager_init() {
        use tempfile::tempdir;
        
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().to_path_buf();
        let database_url = "sqlite::memory:";
        
        let file_manager = storage::FileManager::new(database_url, storage_path).await;
        assert!(file_manager.is_ok());
    }

    #[tokio::test]
    async fn test_file_record_operations() {
        use tempfile::tempdir;
        use chrono::Utc;
        use uuid::Uuid;
        
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().to_path_buf();
        let database_url = "sqlite::memory:";
        
        let file_manager = storage::FileManager::new(database_url, storage_path).await.unwrap();
        
        let file_record = storage::FileRecord {
            id: Uuid::new_v4().to_string(),
            original_name: "test.txt".to_string(),
            stored_name: "stored_test.txt".to_string(),
            file_path: "/tmp/stored_test.txt".to_string(),
            file_size: 1024,
            mime_type: "text/plain".to_string(),
            upload_time: Utc::now(),
            is_video: false,
            thumbnail_path: None,
            video_duration: None,
            video_resolution: None,
        };
        
        assert!(file_manager.save_file_record(&file_record).await.is_ok());
        
        let retrieved = file_manager.get_file_by_id(&file_record.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().original_name, "test.txt");
        
        let files = file_manager.list_files(Some(10), Some(0)).await.unwrap();
        assert_eq!(files.len(), 1);
        
        let stats = file_manager.get_file_stats().await.unwrap();
        assert_eq!(stats.total_files, 1);
        assert_eq!(stats.total_size, 1024);
        assert_eq!(stats.video_count, 0);
    }

    #[test]
    fn test_generate_stored_name() {
        use tempfile::tempdir;
        
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().to_path_buf();
        let database_url = "sqlite::memory:";
        
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let file_manager = storage::FileManager::new(database_url, storage_path).await.unwrap();
            
            let stored_name = file_manager.generate_stored_name("test.txt");
            assert!(stored_name.ends_with(".txt"));
            assert!(stored_name.len() > 10);
            
            let stored_name_no_ext = file_manager.generate_stored_name("test");
            assert!(!stored_name_no_ext.contains("."));
            assert!(stored_name_no_ext.len() > 10);
        });
    }

    #[tokio::test]
    async fn test_server_config_validation() {
        let config = Config::default();
        
        // 验证默认配置是有效的
        let server_address = config.server_address();
        assert_eq!(server_address, "0.0.0.0:3000");
        
        // 验证数据库URL
        let db_url = config.database.database_url();
        assert_eq!(db_url, "sqlite:./files.db");
        
        // 验证存储配置
        assert!(config.storage.max_file_size > 0);
        assert!(config.storage.chunk_size > 0);
    }

    #[tokio::test]
    async fn test_api_response_structure() {
        use crate::server::ApiResponse;
        
        let success_response: ApiResponse<String> = ApiResponse::success("test data".to_string());
        assert!(success_response.success);
        assert!(success_response.data.is_some());
        assert!(success_response.error.is_none());
        
        let error_response: ApiResponse<()> = ApiResponse::error("test error".to_string());
        assert!(!error_response.success);
        assert!(error_response.data.is_none());
        assert!(error_response.error.is_some());
    }
}
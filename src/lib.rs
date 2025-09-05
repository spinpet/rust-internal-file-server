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
        let file_manager = storage::FileManager::new();
        assert!(file_manager.init().await.is_ok());
    }
}
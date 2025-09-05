use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::{Result, ServerError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    pub video: VideoConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_address")]
    pub address: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,
    #[serde(default = "default_request_timeout")]
    pub request_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_database_url")]
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_storage_path")]
    pub path: PathBuf,
    #[serde(default = "default_storage_path")]
    pub upload_dir: PathBuf,
    #[serde(default = "default_max_file_size")]
    pub max_file_size: u64,
    #[serde(default = "default_chunk_size")]
    pub chunk_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    #[serde(default = "default_thumbnail_size")]
    pub thumbnail_size: String,
    #[serde(default = "default_supported_formats")]
    pub supported_formats: Vec<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        // 从默认配置开始
        let default_config = Config::default();
        
        let settings = config::Config::builder()
            // 首先加载默认值
            .add_source(config::Config::try_from(&default_config).map_err(ServerError::from)?)
            .add_source(config::Environment::with_prefix("FILE_SERVER").separator("_"))
            .add_source(config::File::with_name("config.toml").required(false))
            .build()
            .map_err(ServerError::from)?;

        let config: Config = settings.try_deserialize().map_err(ServerError::from)?;
        
        // 验证配置
        config.validate()?;
        
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        // 验证服务器配置
        if self.server.port == 0 {
            return Err(ServerError::validation("端口号不能为0"));
        }

        // 验证存储路径
        if !self.storage.path.exists() {
            std::fs::create_dir_all(&self.storage.path)
                .map_err(|e| ServerError::validation(format!("无法创建存储目录: {}", e)))?;
        }

        // 验证最大文件大小
        if self.storage.max_file_size == 0 {
            return Err(ServerError::validation("最大文件大小不能为0"));
        }

        Ok(())
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.address, self.server.port)
    }
}

impl DatabaseConfig {
    pub fn database_url(&self) -> String {
        self.url.clone()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            storage: StorageConfig::default(),
            video: VideoConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: default_address(),
            port: default_port(),
            max_body_size: default_max_body_size(),
            request_timeout: default_request_timeout(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: default_database_url(),
            max_connections: default_max_connections(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            path: default_storage_path(),
            upload_dir: default_storage_path(),
            max_file_size: default_max_file_size(),
            chunk_size: default_chunk_size(),
        }
    }
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            thumbnail_size: default_thumbnail_size(),
            supported_formats: default_supported_formats(),
        }
    }
}

// 默认值函数
fn default_address() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_max_body_size() -> usize {
    10 * 1024 * 1024 * 1024 // 10GB
}

fn default_request_timeout() -> u64 {
    300 // 5分钟
}

fn default_database_url() -> String {
    "sqlite:./files.db".to_string()
}

fn default_max_connections() -> u32 {
    10
}

fn default_storage_path() -> PathBuf {
    PathBuf::from("./storage")
}

fn default_max_file_size() -> u64 {
    10 * 1024 * 1024 * 1024 // 10GB
}

fn default_chunk_size() -> usize {
    8 * 1024 * 1024 // 8MB
}

fn default_thumbnail_size() -> String {
    "320x240".to_string()
}

fn default_supported_formats() -> Vec<String> {
    vec![
        "mp4".to_string(),
        "avi".to_string(),
        "mkv".to_string(),
        "mov".to_string(),
        "wmv".to_string(),
        "flv".to_string(),
        "webm".to_string(),
    ]
}
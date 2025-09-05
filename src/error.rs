use thiserror::Error;

pub type Result<T> = std::result::Result<T, ServerError>;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("配置错误: {0}")]
    Config(#[from] config::ConfigError),

    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("序列化错误: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("HTTP错误: {0}")]
    Http(#[from] hyper::Error),

    #[error("Axum错误: {0}")]
    Axum(String),

    #[error("文件操作错误: {message}")]
    FileOperation { message: String },

    #[error("视频处理错误: {message}")]
    VideoProcessing { message: String },

    #[error("验证错误: {message}")]
    Validation { message: String },

    #[error("未找到资源: {resource}")]
    NotFound { resource: String },

    #[error("权限不足: {action}")]
    PermissionDenied { action: String },

    #[error("内部服务器错误: {0}")]
    Internal(#[from] anyhow::Error),
}

impl ServerError {
    pub fn file_operation(message: impl Into<String>) -> Self {
        Self::FileOperation {
            message: message.into(),
        }
    }

    pub fn video_processing(message: impl Into<String>) -> Self {
        Self::VideoProcessing {
            message: message.into(),
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    pub fn permission_denied(action: impl Into<String>) -> Self {
        Self::PermissionDenied {
            action: action.into(),
        }
    }
}

// Axum 错误转换
impl From<axum::Error> for ServerError {
    fn from(err: axum::Error) -> Self {
        Self::Axum(err.to_string())
    }
}

// 响应状态码映射
impl ServerError {
    pub fn status_code(&self) -> u16 {
        match self {
            Self::NotFound { .. } => 404,
            Self::Validation { .. } => 400,
            Self::PermissionDenied { .. } => 403,
            Self::Config(_) | Self::Axum(_) => 500,
            Self::Database(_) | Self::Io(_) => 500,
            Self::Serde(_) | Self::Http(_) => 500,
            Self::FileOperation { .. } => 500,
            Self::VideoProcessing { .. } => 500,
            Self::Internal(_) => 500,
        }
    }
}
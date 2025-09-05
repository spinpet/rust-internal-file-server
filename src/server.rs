use crate::config::Config;
use crate::error::ServerError;
use axum::{
    Router,
    response::Json,
    http::StatusCode,
    routing::get,
};
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;
use tracing::{info, error};

type Result<T> = std::result::Result<T, ServerError>;

pub async fn start_server(config: Config) -> Result<()> {
    let address = config.server_address();
    
    // 构建路由
    let app = create_router(config.clone()).await?;

    info!("服务器启动在: http://{}", address);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .map_err(|e| ServerError::Io(e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| ServerError::Internal(e.into()))?;

    Ok(())
}

async fn create_router(config: Config) -> Result<Router> {
    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/api/info", get(server_info))
        .layer(CorsLayer::permissive());

    Ok(app)
}

// 健康检查端点
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "rust-internal-file-server",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// 服务器信息端点
async fn server_info() -> Json<Value> {
    Json(json!({
        "name": "Rust Internal File Server",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "基于Rust的内网文件共享服务器，支持大文件存储和在线视频播放",
        "features": [
            "大文件上传下载",
            "视频在线播放",
            "断点续传",
            "文件管理"
        ]
    }))
}
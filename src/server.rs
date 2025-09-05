use crate::config::Config;
use crate::error::ServerError;
use crate::storage::FileManager;
use axum::{
    Router,
    response::Json,
    routing::get,
    extract::{Query, Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, error};

type Result<T> = std::result::Result<T, ServerError>;

#[derive(Clone)]
pub struct AppState {
    pub file_manager: Arc<FileManager>,
    pub config: Config,
}

pub async fn start_server(config: Config) -> Result<()> {
    let address = config.server_address();
    
    // 创建文件管理器
    let file_manager = Arc::new(
        FileManager::new(
            &config.database.database_url(),
            config.storage.upload_dir.clone(),
        ).await?
    );
    
    // 创建应用状态
    let state = AppState {
        file_manager,
        config: config.clone(),
    };
    
    // 构建路由
    let app = create_router(state).await?;

    info!("服务器启动在: http://{}", address);
    info!("数据库: {}", config.database.database_url());
    info!("存储目录: {:?}", config.storage.upload_dir);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .map_err(|e| ServerError::Io(e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| ServerError::Internal(e.into()))?;

    Ok(())
}

async fn create_router(state: AppState) -> Result<Router> {
    let app = Router::new()
        // 健康检查和信息接口
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/api/info", get(server_info))
        
        // 文件管理 API
        .route("/api/files", get(list_files))
        .route("/api/files/:file_id", get(get_file_info))
        .route("/api/files/:file_id", axum::routing::delete(delete_file))
        .route("/api/stats", get(get_file_stats))
        
        // 静态文件服务 (将在后续任务中实现)
        .route("/files/*path", get(serve_file))
        
        // 中间件
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

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

// 查询参数结构
#[derive(Deserialize)]
struct ListFilesQuery {
    limit: Option<i32>,
    offset: Option<i32>,
}

// API响应结构
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

// 文件列表接口
async fn list_files(
    Query(params): Query<ListFilesQuery>,
    State(state): State<AppState>,
) -> std::result::Result<Json<ApiResponse<Vec<crate::storage::FileRecord>>>, (StatusCode, Json<ApiResponse<()>>)> {
    match state.file_manager.list_files(params.limit, params.offset).await {
        Ok(files) => Ok(Json(ApiResponse::success(files))),
        Err(e) => {
            error!("获取文件列表失败: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(format!("获取文件列表失败: {}", e)))
            ))
        }
    }
}

// 获取单个文件信息
async fn get_file_info(
    Path(file_id): Path<String>,
    State(state): State<AppState>,
) -> std::result::Result<Json<ApiResponse<crate::storage::FileRecord>>, (StatusCode, Json<ApiResponse<()>>)> {
    match state.file_manager.get_file_by_id(&file_id).await {
        Ok(Some(file)) => Ok(Json(ApiResponse::success(file))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(format!("文件不存在: {}", file_id)))
        )),
        Err(e) => {
            error!("获取文件信息失败: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(format!("获取文件信息失败: {}", e)))
            ))
        }
    }
}

// 删除文件
async fn delete_file(
    Path(file_id): Path<String>,
    State(state): State<AppState>,
) -> std::result::Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    match state.file_manager.delete_file(&file_id).await {
        Ok(true) => Ok(Json(ApiResponse::success(()))),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(format!("文件不存在: {}", file_id)))
        )),
        Err(e) => {
            error!("删除文件失败: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(format!("删除文件失败: {}", e)))
            ))
        }
    }
}

// 获取文件统计信息
async fn get_file_stats(
    State(state): State<AppState>,
) -> std::result::Result<Json<ApiResponse<crate::storage::FileStats>>, (StatusCode, Json<ApiResponse<()>>)> {
    match state.file_manager.get_file_stats().await {
        Ok(stats) => Ok(Json(ApiResponse::success(stats))),
        Err(e) => {
            error!("获取统计信息失败: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(format!("获取统计信息失败: {}", e)))
            ))
        }
    }
}

// 文件服务接口 (占位符)
async fn serve_file(
    Path(_path): Path<String>,
) -> std::result::Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(ApiResponse::error("文件服务功能将在后续任务中实现".to_string()))
    ))
}
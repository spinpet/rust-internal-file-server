use rust_internal_file_server::config::Config;
use rust_internal_file_server::server::start_server;
use rust_internal_file_server::Result;
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    info!("启动内网文件服务器...");

    // 加载配置
    let config = Config::load()?;
    info!("配置加载完成: {}", config.server.address);

    // 启动服务器
    start_server(config).await?;

    Ok(())
}
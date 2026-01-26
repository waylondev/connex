use tracing_subscriber::{fmt, EnvFilter};

/// 统一的日志初始化方法
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量读取日志级别，默认info
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
        
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .init();
        
    Ok(())
}

/// 开发环境日志配置
pub fn init_dev() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .pretty()
        .init();
    Ok(())
}

/// 生产环境日志配置
pub fn init_prod() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .json()
        .init();
    Ok(())
}

/// 测试环境日志配置
pub fn init_test() -> Result<(), Box<dyn std::error::Error>> {
    // 测试环境默认不输出日志，避免干扰测试输出
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .try_init();
    Ok(())
}
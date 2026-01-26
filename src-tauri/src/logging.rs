
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

/// 智能日志配置 - 根据环境变量自动选择最佳配置
pub fn init_smart() -> Result<(), Box<dyn std::error::Error>> {
    // 优先使用用户指定的环境变量
    if let Ok(log_type) = std::env::var("CONNEX_LOG_TYPE") {
        match log_type.as_str() {
            "dev" => return init_dev(),
            "prod" => return init_prod(),
            "test" => return init_test(),
            _ => {} // 使用默认配置
        }
    }
    
    // 如果没有指定，根据构建模式智能选择
    if cfg!(debug_assertions) {
        // 调试模式：使用开发环境配置
        init_dev()
    } else {
        // 发布模式：使用生产环境配置
        init_prod()
    }
}
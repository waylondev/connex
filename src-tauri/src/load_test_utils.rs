use crate::load_test::Config;
use crate::stats::LoadTestResult;

/// 打印测试参数的辅助方法 - 负载测试特有
pub fn print_test_config(config: &Config) {
    tracing::info!("开始负载测试: URL={}, 并发数={}, 测试时长={:?}", 
                   config.url, config.concurrency, config.duration);
}

/// 打印测试结果的辅助方法 - 负载测试特有
pub fn print_test_result(result: &LoadTestResult) {
    tracing::info!(
        "测试结果: 总请求数={}, 成功={}, 失败={}, RPS={:.2}, 平均延迟={}ms, 错误统计={:?}",
        result.total_requests,
        result.successful_requests,
        result.failed_requests,
        result.requests_per_second,
        result.average_latency,
        result.error_stats
    );
}

/// 创建优化的HTTP客户端 - 支持高并发
pub fn create_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        // 优化连接池设置 - 针对高并发优化
        .pool_max_idle_per_host(1000)  // 大幅增加空闲连接数支持更高并发
        .pool_idle_timeout(Some(std::time::Duration::from_secs(30)))  // 延长空闲超时
        // 调整超时设置 - 更合理的优化
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(5))
        // 启用TCP_NODELAY，减少延迟
        .tcp_nodelay(true)
        // 启用HTTP/1.1标题大小写转换，兼容性更好
        .http1_title_case_headers()
        // 允许HTTP/2协商，但不强制
        .http2_adaptive_window(true)
        // 禁用自动重定向
        .redirect(reqwest::redirect::Policy::none())
        // 禁用压缩，减少CPU开销
        .no_gzip()
        .no_brotli()
        .no_deflate()
        // 启用连接复用
        .build()
        .expect("Failed to create HTTP client")
}



/// 默认并发数 - 负载测试特有
pub fn default_concurrency() -> usize {
    10
}



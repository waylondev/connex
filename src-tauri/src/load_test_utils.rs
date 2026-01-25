use std::sync::{Arc, atomic::{AtomicU32, AtomicU64}};
use std::time::Duration;
use reqwest;
use crate::load_test::{Config, LoadTestResult};

/// 打印测试参数的辅助方法 - 负载测试特有
pub fn print_test_config(config: &Config) {
    println!("开始负载测试:");
    println!("URL: {}", config.url);
    println!("并发数: {}", config.concurrency);
    println!("测试时长: {:?}", config.duration);
}

/// 打印测试结果的辅助方法 - 负载测试特有
pub fn print_test_result(result: &LoadTestResult) {
    println!("测试结果:");
    println!("总请求数: {}", result.total_requests);
    println!("成功请求数: {}", result.successful_requests);
    println!("失败请求数: {}", result.failed_requests);
    println!("每秒请求数: {:.2}", result.requests_per_second);
    println!("平均延迟: {}ms", result.average_latency);
    println!("错误统计: {:?}", result.error_stats);
}

/// 打印监控数据的辅助方法 - 负载测试特有
pub fn print_monitoring_data(metrics: &crate::monitoring::RealTimeMetrics) {
    println!("\n监控数据:");
    println!("每秒请求数: {:.2} RPS", metrics.rps);
    println!("延迟分布:");
    println!("  P50: {}ms", metrics.latency_percentiles.p50);
    println!("  P90: {}ms", metrics.latency_percentiles.p90);
    println!("  P95: {}ms", metrics.latency_percentiles.p95);
    println!("  P99: {}ms", metrics.latency_percentiles.p99);
}

/// 创建优化的HTTP客户端 - 支持高并发
pub fn create_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        // 调整连接池大小，由操作系统限制
        .pool_max_idle_per_host(usize::MAX)
        // 调整超时设置，适合高并发场景
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(10))
        // 启用TCP_NODELAY，减少延迟
        .tcp_nodelay(true)
        // 启用HTTP/1.1标题大小写转换
        .http1_title_case_headers()
        // 禁用自动重定向，减少不必要的请求
        .redirect(reqwest::redirect::Policy::none())
        // 禁用压缩，减少CPU开销
        .no_gzip()
        .no_brotli()
        .no_deflate()
        .build()
        .expect("Failed to create HTTP client")
}



/// 默认并发数 - 负载测试特有
pub fn default_concurrency() -> usize {
    10
}



/// 计算并生成测试结果 - 负载测试特有
pub fn generate_test_result(
    start_time: std::time::Instant,
    successful: &Arc<AtomicU32>,
    failed: &Arc<AtomicU32>,
    total_latency: &Arc<AtomicU64>,
    connection_errors: &Arc<AtomicU32>,
    timeout_errors: &Arc<AtomicU32>,
    http_errors: &Arc<AtomicU32>,
    other_errors: &Arc<AtomicU32>,
) -> crate::load_test::LoadTestResult {
    use std::sync::atomic::Ordering;
    
    let elapsed = start_time.elapsed();
    let total_successful = successful.load(Ordering::Relaxed);
    let total_failed = failed.load(Ordering::Relaxed);
    let total_requests = total_successful + total_failed;
    
    let avg_latency = if total_successful > 0 {
        // total_latency已经是毫秒值
        let total_latency_ms = total_latency.load(Ordering::Relaxed);
        total_latency_ms / total_successful as u64
    } else {
        0
    };
    
    let rps = total_requests as f64 / elapsed.as_secs_f64();
    
    // 收集错误统计
    let error_stats = crate::load_test::ErrorStats {
        connection_errors: connection_errors.load(Ordering::Relaxed),
        timeout_errors: timeout_errors.load(Ordering::Relaxed),
        http_errors: http_errors.load(Ordering::Relaxed),
        other_errors: other_errors.load(Ordering::Relaxed),
    };
    
    // 创建测试结果
    crate::load_test::LoadTestResult {
        total_requests,
        successful_requests: total_successful,
        failed_requests: total_failed,
        requests_per_second: rps,
        average_latency: avg_latency,
        error_stats,
    }
}

use std::sync::{Arc, Mutex, atomic::{AtomicU32, Ordering}};
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

/// 创建优化的HTTP客户端 - 支持高并发
pub fn create_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        // 增加每个主机的最大空闲连接数
        .pool_max_idle_per_host(500)
        // 调整超时设置，适合长连接
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
}

/// 处理单个HTTP请求的辅助方法 - 负载测试特有
pub async fn process_single_request(
    client: Arc<reqwest::Client>,
    url: Arc<String>,
    successful: Arc<AtomicU32>,
    failed: Arc<AtomicU32>,
    total_latency: Arc<Mutex<Duration>>,
    connection_errors: Arc<AtomicU32>,
    timeout_errors: Arc<AtomicU32>,
    http_errors: Arc<AtomicU32>,
    other_errors: Arc<AtomicU32>,
) {
    let request_start = std::time::Instant::now();
    
    match client.get(url.as_str()).send().await {
        Ok(response) => {
            // 检查HTTP状态码
            if response.status().is_success() {
                successful.fetch_add(1, Ordering::Relaxed);
                let latency = request_start.elapsed();
                let mut guard = total_latency.lock().unwrap();
                *guard += latency;
            } else {
                failed.fetch_add(1, Ordering::Relaxed);
                http_errors.fetch_add(1, Ordering::Relaxed);
            }
        }
        Err(e) => {
            failed.fetch_add(1, Ordering::Relaxed);
            
            // 分类统计错误类型
            if e.is_connect() {
                connection_errors.fetch_add(1, Ordering::Relaxed);
            } else if e.is_timeout() {
                timeout_errors.fetch_add(1, Ordering::Relaxed);
            } else {
                other_errors.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

/// 默认并发数 - 负载测试特有
pub fn default_concurrency() -> usize {
    10
}

/// 默认测试时长 - 负载测试特有
pub fn default_duration() -> Duration {
    Duration::from_secs(10)
}

/// 计算并生成测试结果 - 负载测试特有
pub fn generate_test_result(
    start_time: std::time::Instant,
    successful: &Arc<AtomicU32>,
    failed: &Arc<AtomicU32>,
    total_latency: &Arc<Mutex<Duration>>,
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
        // 将Duration转换为毫秒
        (*total_latency.lock().unwrap() / total_successful).as_millis() as u64
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

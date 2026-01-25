use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use futures::stream::{self, StreamExt};

/// 负载测试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub url: String,
    #[serde(default = "default_concurrency")]
    pub concurrency: usize, // 默认10
    #[serde(default = "default_duration")]
    pub duration: Duration, // 默认10秒
    #[serde(default = "default_batch_size")]
    pub batch_size: usize, // 默认1
}

/// 默认并发数
fn default_concurrency() -> usize {
    10
}

/// 默认测试时长
fn default_duration() -> Duration {
    Duration::from_secs(10)
}

/// 默认批量大小
fn default_batch_size() -> usize {
    1
}

/// 错误类型统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStats {
    pub connection_errors: u32,
    pub timeout_errors: u32,
    pub http_errors: u32,
    pub other_errors: u32,
}

/// 负载测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTestResult {
    pub total_requests: u32,
    pub successful_requests: u32,
    pub failed_requests: u32,
    pub requests_per_second: f64,
    pub average_latency: Duration,
    pub error_stats: ErrorStats, // 详细的错误统计
}

/// 创建基于时间的请求流 - 持续生成任务直到测试时间结束
fn create_request_stream(end_time: std::time::Instant) -> impl futures::Stream<Item = usize> {
    let mut count = 0;
    stream::iter(std::iter::from_fn(move || {
        if std::time::Instant::now() < end_time {
            count += 1;
            Some(count)
        } else {
            None
        }
    }))
}

/// 请求处理策略 - 统一使用批量处理，支持单条和批量
async fn process_requests(
    client: Arc<reqwest::Client>,
    url: Arc<String>,
    successful: Arc<std::sync::atomic::AtomicU32>,
    failed: Arc<std::sync::atomic::AtomicU32>,
    total_latency: Arc<std::sync::Mutex<Duration>>,
    connection_errors: Arc<std::sync::atomic::AtomicU32>,
    timeout_errors: Arc<std::sync::atomic::AtomicU32>,
    http_errors: Arc<std::sync::atomic::AtomicU32>,
    other_errors: Arc<std::sync::atomic::AtomicU32>,
    end_time: std::time::Instant,
    batch_size: usize, // 批量大小，1表示单条处理
) {
    // 检查是否超过测试时间
    if std::time::Instant::now() >= end_time {
        return;
    }
    
    // 统一使用批量处理逻辑，batch_size=1就是单条处理
    let request_start = std::time::Instant::now();
    
    // 创建批量请求
    let futures: Vec<_> = (0..batch_size)
        .map(|_| client.get(url.as_str()).send())
        .collect();
    
    // 批量等待所有请求完成
    let results = futures::future::join_all(futures).await;
    
    // 处理批量结果
    for result in results {
        match result {
            Ok(response) => {
                // 检查HTTP状态码
                if response.status().is_success() {
                    successful.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let latency = request_start.elapsed();
                    let mut guard = total_latency.lock().unwrap();
                    *guard += latency;
                } else {
                    failed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    http_errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }
            Err(e) => {
                failed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                
                // 分类统计错误类型
                if e.is_connect() {
                    connection_errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                } else if e.is_timeout() {
                    timeout_errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                } else {
                    other_errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }
    }
}

// 直接使用serde默认值，不需要单独的配置处理函数

/// 执行负载测试 - 使用流式处理+buffer_unordered实现真正的高并发
pub async fn run(config: Config) -> LoadTestResult {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Mutex;
    
    // 打印负载测试参数
    println!("开始负载测试:");
    println!("URL: {}", config.url);
    println!("并发数: {}", config.concurrency);
    println!("测试时长: {:?}", config.duration);
    println!("批量大小: {}", config.batch_size);
    
    let client = Arc::new(reqwest::Client::new());
    let url = Arc::new(config.url.clone());
    
    // 使用Arc包装原子变量
    let successful = Arc::new(AtomicU32::new(0));
    let failed = Arc::new(AtomicU32::new(0));
    let total_latency = Arc::new(Mutex::new(Duration::default()));
    
    // 错误类型统计
    let connection_errors = Arc::new(AtomicU32::new(0));
    let timeout_errors = Arc::new(AtomicU32::new(0));
    let http_errors = Arc::new(AtomicU32::new(0));
    let other_errors = Arc::new(AtomicU32::new(0));
    
    let start_time = std::time::Instant::now();
    let end_time = start_time + config.duration;
    
    // 使用流式处理实现真正的高并发
    let request_stream = create_request_stream(end_time);
    
    let _results: Vec<()> = request_stream
        .map(|_| {
            let client = Arc::clone(&client);
            let url = Arc::clone(&url);
            let successful = Arc::clone(&successful);
            let failed = Arc::clone(&failed);
            let total_latency = Arc::clone(&total_latency);
            let connection_errors = Arc::clone(&connection_errors);
            let timeout_errors = Arc::clone(&timeout_errors);
            let http_errors = Arc::clone(&http_errors);
            let other_errors = Arc::clone(&other_errors);
            let end_time = end_time;
            
            async move {
                process_requests(
                    client, url, successful, failed, total_latency,
                    connection_errors, timeout_errors, http_errors, other_errors,
                    end_time, config.batch_size
                ).await
            }
        })
        .buffer_unordered(config.concurrency) // 关键：控制并发度，实现自动背压
        .collect()
        .await;
    
    let elapsed = start_time.elapsed();
    let total_successful = successful.load(Ordering::Relaxed);
    let total_failed = failed.load(Ordering::Relaxed);
    let total_requests = total_successful + total_failed;
    
    let avg_latency = if total_successful > 0 {
        *total_latency.lock().unwrap() / total_successful
    } else {
        Duration::default()
    };
    
    let rps = total_requests as f64 / elapsed.as_secs_f64();
    
    // 收集错误统计
    let error_stats = ErrorStats {
        connection_errors: connection_errors.load(Ordering::Relaxed),
        timeout_errors: timeout_errors.load(Ordering::Relaxed),
        http_errors: http_errors.load(Ordering::Relaxed),
        other_errors: other_errors.load(Ordering::Relaxed),
    };
    
    LoadTestResult {
        total_requests,
        successful_requests: total_successful,
        failed_requests: total_failed,
        requests_per_second: rps,
        average_latency: avg_latency,
        error_stats,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_load_test() {
        let config = Config {
            url: "http://httpbin.org/get".to_string(),
            concurrency: 10,
            duration: Duration::from_secs(10),
            batch_size: 1,
        };
        
        let result = run(config).await;
        
        assert!(result.total_requests > 0);
        assert!(result.requests_per_second > 0.0);
        println!("测试结果: {:?}", result);
    }
}
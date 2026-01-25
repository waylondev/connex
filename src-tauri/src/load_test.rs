use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

// 导入模块：负载测试特有方法
use crate::load_test_utils;

/// 负载测试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub url: String,
    #[serde(default = "load_test_utils::default_concurrency")]
    pub concurrency: usize, // 默认10
    #[serde(default = "load_test_utils::default_duration")]
    pub duration: Duration, // 默认10秒
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
    pub average_latency: u64, // 毫秒
    pub error_stats: ErrorStats, // 详细的错误统计
}



// 直接使用serde默认值，不需要单独的配置处理函数

/// 执行负载测试 - 使用spawn直接创建task实现高并发
pub async fn run(config: Config) -> LoadTestResult {
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::AtomicU64;
    
    // 打印负载测试参数
    load_test_utils::print_test_config(&config);
    
    // 使用优化的HTTP客户端，支持高并发
    let client = Arc::new(load_test_utils::create_http_client());
    let url = Arc::new(config.url.clone());
    
    // 使用Arc包装原子变量
    let successful = Arc::new(AtomicU32::new(0));
    let failed = Arc::new(AtomicU32::new(0));
    // 使用AtomicU64存储总延迟（毫秒），避免Mutex竞争
    let total_latency = Arc::new(AtomicU64::new(0));
    
    // 错误类型统计
    let connection_errors = Arc::new(AtomicU32::new(0));
    let timeout_errors = Arc::new(AtomicU32::new(0));
    let http_errors = Arc::new(AtomicU32::new(0));
    let other_errors = Arc::new(AtomicU32::new(0));
    
    let start_time = std::time::Instant::now();
    let end_time = start_time + config.duration;
    
    // 创建指定数量的并发task
    let mut tasks = Vec::with_capacity(config.concurrency);
    for _ in 0..config.concurrency {
        // 克隆所有需要的Arc引用
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
        
        // 直接spawn task，每个task持续发送请求直到测试结束
        let task = tokio::spawn(async move {
            // 每个task内部是串行发送请求的循环
            while std::time::Instant::now() < end_time {
                // 发送单个请求并更新统计
                let request_start = std::time::Instant::now();
                
                match client.get(url.as_str()).send().await {
                    Ok(response) => {
                        // 检查HTTP状态码
                        if response.status().is_success() {
                            successful.fetch_add(1, Ordering::Relaxed);
                            // 计算延迟并更新原子变量（毫秒）
                            let latency = request_start.elapsed().as_millis() as u64;
                            total_latency.fetch_add(latency, Ordering::Relaxed);
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
        });
        
        tasks.push(task);
    }
    
    // 等待所有task完成
    for task in tasks {
        task.await.unwrap();
    }
    
    // 生成测试结果
    let result = load_test_utils::generate_test_result(
        start_time, &successful, &failed, &total_latency,
        &connection_errors, &timeout_errors, &http_errors, &other_errors
    );
    
    // 调用辅助方法打印测试结果
    load_test_utils::print_test_result(&result);
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_load_test() {
        let config = Config {
            url: "http://localhost:3000".to_string(),
            concurrency: 1000000,
            duration: Duration::from_secs(10),
        };
        
        let result = run(config).await;
        
        assert!(result.total_requests > 0);
        assert!(result.requests_per_second > 0.0);
    }
}
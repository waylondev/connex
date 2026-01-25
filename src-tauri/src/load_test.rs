use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use futures::stream::StreamExt;
use reqwest;

// 导入模块：公共方法
use crate::utils;
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

/// 执行负载测试 - 使用流式处理+buffer_unordered实现真正的高并发
pub async fn run(config: Config) -> LoadTestResult {
    use std::sync::atomic::AtomicU32;
    use std::sync::Mutex;
    
    // 打印负载测试参数
    load_test_utils::print_test_config(&config);
    
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
    let request_stream = utils::create_request_stream(end_time);
    
    let _results: Vec<()> = request_stream
        .map(|_| {
            // 必须克隆Arc引用，因为每个异步任务需要拥有自己的引用
            // Arc::clone()是轻量级操作，仅增加引用计数
            let client = Arc::clone(&client);
            let url = Arc::clone(&url);
            let successful = Arc::clone(&successful);
            let failed = Arc::clone(&failed);
            let total_latency = Arc::clone(&total_latency);
            let connection_errors = Arc::clone(&connection_errors);
            let timeout_errors = Arc::clone(&timeout_errors);
            let http_errors = Arc::clone(&http_errors);
            let other_errors = Arc::clone(&other_errors);
            
            async move {
                // 调用load_test_utils模块的process_single_request函数处理单个请求
                load_test_utils::process_single_request(
                    client, url, successful, failed, total_latency,
                    connection_errors, timeout_errors, http_errors, other_errors
                ).await;
            }
        })
        .buffer_unordered(config.concurrency) // 直接控制HTTP请求并发度
        .collect()
        .await;
    
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
            url: "http://httpbin.org/get".to_string(),
            concurrency: 100,
            duration: Duration::from_secs(10),
        };
        
        let result = run(config).await;
        
        assert!(result.total_requests > 0);
        assert!(result.requests_per_second > 0.0);
    }
}
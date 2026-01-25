use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
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



/// 配置类状态：测试过程中不会改变
struct TestConfig {
    client: Arc<reqwest::Client>,
    url: Arc<String>,
}

/// 统计类状态：测试过程中会被更新
struct TestStatistics {
    successful: Arc<AtomicU32>,
    failed: Arc<AtomicU32>,
    total_latency: Arc<AtomicU64>,
    connection_errors: Arc<AtomicU32>,
    timeout_errors: Arc<AtomicU32>,
    http_errors: Arc<AtomicU32>,
    other_errors: Arc<AtomicU32>,
}

impl TestStatistics {
    /// 批量更新统计数据，减少原子操作开销
    fn update_statistics(
        &self,
        local_successful: &mut u32,
        local_failed: &mut u32,
        local_latency: &mut u64,
        local_connection_errors: &mut u32,
        local_timeout_errors: &mut u32,
        local_http_errors: &mut u32,
        local_other_errors: &mut u32,
    ) {
        // 批量更新原子变量，减少CPU cache失效
        self.successful.fetch_add(*local_successful, Ordering::Relaxed);
        self.failed.fetch_add(*local_failed, Ordering::Relaxed);
        self.total_latency.fetch_add(*local_latency, Ordering::Relaxed);
        self.connection_errors.fetch_add(*local_connection_errors, Ordering::Relaxed);
        self.timeout_errors.fetch_add(*local_timeout_errors, Ordering::Relaxed);
        self.http_errors.fetch_add(*local_http_errors, Ordering::Relaxed);
        self.other_errors.fetch_add(*local_other_errors, Ordering::Relaxed);
        
        // 重置本地计数器
        *local_successful = 0;
        *local_failed = 0;
        *local_latency = 0;
        *local_connection_errors = 0;
        *local_timeout_errors = 0;
        *local_http_errors = 0;
        *local_other_errors = 0;
    }
}

/// 测试状态：组合配置和统计
struct TestState {
    config: Arc<TestConfig>,
    stats: Arc<TestStatistics>,
}

/// 辅助函数：初始化测试状态
fn initialize_test_state(config: &Config) -> (Arc<TestState>, std::time::Instant, std::time::Instant) {
    // 使用优化的HTTP客户端，支持高并发
    let client = Arc::new(load_test_utils::create_http_client());
    let url = Arc::new(config.url.clone());
    
    // 配置类状态
    let test_config = Arc::new(TestConfig {
        client,
        url,
    });
    
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
    
    // 统计类状态
    let test_stats = Arc::new(TestStatistics {
        successful,
        failed,
        total_latency,
        connection_errors,
        timeout_errors,
        http_errors,
        other_errors,
    });
    
    // 组合状态
    let test_state = Arc::new(TestState {
        config: test_config,
        stats: test_stats,
    });
    
    let start_time = std::time::Instant::now();
    let end_time = start_time + config.duration;
    
    (test_state, start_time, end_time)
}

/// 辅助函数：生成并运行测试任务
fn spawn_test_tasks(
    test_state: &Arc<TestState>,
    end_time: std::time::Instant,
    concurrency: usize
) -> Vec<tokio::task::JoinHandle<()>> {
    let mut tasks = Vec::with_capacity(concurrency);
    
    for _ in 0..concurrency {
        let state = Arc::clone(test_state);
        let end_time = end_time;
        
        // 直接spawn task，每个task持续发送请求直到测试结束
        let task = tokio::spawn(async move {
            // 批量更新统计数据，减少原子操作开销
            let mut local_successful = 0;
            let mut local_failed = 0;
            let mut local_latency = 0u64;
            let mut local_connection_errors = 0;
            let mut local_timeout_errors = 0;
            let mut local_http_errors = 0;
            let mut local_other_errors = 0;
            
            // 使用时间检查任务是否需要退出
            while std::time::Instant::now() < end_time {
                // 发送单个请求
                let request_start = std::time::Instant::now();
                
                match state.config.client.get(state.config.url.as_str()).send().await {
                    Ok(response) => {
                        // 检查HTTP状态码
                        if response.status().is_success() {
                            local_successful += 1;
                            // 计算延迟并更新本地变量（毫秒）
                            let latency = request_start.elapsed().as_millis() as u64;
                            local_latency += latency;
                        } else {
                            local_failed += 1;
                            local_http_errors += 1;
                        }
                    }
                    Err(e) => {
                        local_failed += 1;
                        
                        // 分类统计错误类型
                        if e.is_connect() {
                            local_connection_errors += 1;
                        } else if e.is_timeout() {
                            local_timeout_errors += 1;
                        } else {
                            local_other_errors += 1;
                        }
                    }
                }
                
                // 每100个请求批量更新一次原子变量，减少原子操作开销
                if local_successful + local_failed >= 100 {
                    // 调用辅助方法批量更新统计数据
                    state.stats.update_statistics(
                        &mut local_successful,
                        &mut local_failed,
                        &mut local_latency,
                        &mut local_connection_errors,
                        &mut local_timeout_errors,
                        &mut local_http_errors,
                        &mut local_other_errors,
                    );
                }
            }
            
            // 测试结束时，更新剩余的统计数据
            if local_successful + local_failed > 0 {
                // 调用辅助方法批量更新统计数据
                state.stats.update_statistics(
                    &mut local_successful,
                    &mut local_failed,
                    &mut local_latency,
                    &mut local_connection_errors,
                    &mut local_timeout_errors,
                    &mut local_http_errors,
                    &mut local_other_errors,
                );
            }
        });
        
        tasks.push(task);
    }
    
    tasks
}

/// 辅助函数：等待任务完成
async fn wait_for_tasks(tasks: Vec<tokio::task::JoinHandle<()>>) {
    for task in tasks {
        task.await.unwrap();
    }
}

/// 辅助函数：生成测试结果
fn generate_test_result(
    test_state: &Arc<TestState>,
    start_time: std::time::Instant
) -> LoadTestResult {
    let result = load_test_utils::generate_test_result(
        start_time, &test_state.stats.successful, &test_state.stats.failed, &test_state.stats.total_latency,
        &test_state.stats.connection_errors, &test_state.stats.timeout_errors, &test_state.stats.http_errors, &test_state.stats.other_errors
    );
    
    // 调用辅助方法打印测试结果
    load_test_utils::print_test_result(&result);
    
    result
}

/// 执行负载测试 - 使用spawn直接创建task实现高并发
pub async fn run(config: Config) -> LoadTestResult {
    // 打印负载测试参数
    load_test_utils::print_test_config(&config);
    
    // 1. 初始化测试状态
    let (test_state, start_time, end_time) = initialize_test_state(&config);
    
    // 2. 生成并运行测试任务
    let tasks = spawn_test_tasks(&test_state, end_time, config.concurrency);
    
    // 3. 等待任务完成
    wait_for_tasks(tasks).await;
    
    // 4. 生成测试结果
    generate_test_result(&test_state, start_time)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// 简单测试：低并发，短时间，快速运行
    /// 用于开发调试，CI/CD自动运行
    #[tokio::test]
    async fn test_load_test_simple() {
        let config = Config {
            url: "http://httpbin.org/get".to_string(),
            concurrency: 10,
            duration: Duration::from_secs(2),
        };
        
        let result = run(config).await;
        
        assert!(result.total_requests > 0);
        assert!(result.requests_per_second > 0.0);
    }

    /// 高并发测试：高并发，长时间，手动运行
    /// 用于验证极端情况下的性能表现
    #[tokio::test]
    #[ignore] // 默认忽略，需要手动运行
    async fn test_load_test_high_concurrency() {
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
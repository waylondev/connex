use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

// 导入模块：负载测试特有方法
use crate::load_test_utils;
use crate::stats::AsyncStats;
pub use crate::stats::LoadTestResult;

/// 负载测试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub url: String,
    #[serde(default = "load_test_utils::default_concurrency")]
    pub concurrency: usize, // 默认10
    #[serde(default = "default_duration_seconds")]
    pub duration: u64, // 秒数，默认10秒
}

/// 默认测试时长（秒）
pub fn default_duration_seconds() -> u64 {
    10
}



// 直接使用serde默认值，不需要单独的配置处理函数



/// 配置类状态：测试过程中不会改变
struct TestConfig {
    client: Arc<reqwest::Client>,
    url: Arc<String>,
}



/// 测试状态：组合配置和统计
struct TestState {
    config: Arc<TestConfig>,
    stats: Arc<AsyncStats>,
}

/// 类型别名：简化复杂类型
pub type TaskHandle = tokio::task::JoinHandle<()>;
pub type TaskList = Vec<TaskHandle>;

/// 初始化测试配置
fn initialize_config(config: &Config) -> Arc<TestConfig> {
    let client = Arc::new(load_test_utils::create_http_client());
    let url = Arc::new(config.url.clone());
    
    Arc::new(TestConfig {
        client,
        url,
    })
}

/// 初始化测试统计
fn initialize_statistics() -> Arc<AsyncStats> {
    Arc::new(AsyncStats::new())
}

/// 辅助函数：初始化测试状态
fn initialize_test_state(config: &Config) -> (Arc<TestState>, std::time::Instant, std::time::Instant) {
    let test_config = initialize_config(config);
    let stats = initialize_statistics();
    
    let test_state = Arc::new(TestState {
        config: test_config,
        stats,
    });
    
    let start_time = std::time::Instant::now();
    let end_time = start_time + Duration::from_secs(config.duration);
    
    (test_state, start_time, end_time)
}

/// 辅助函数：生成并运行测试任务
fn spawn_test_tasks(
    test_state: &Arc<TestState>,
    end_time: std::time::Instant,
    concurrency: usize
) -> TaskList {
    let mut tasks = Vec::new();
    
    // 优化：限制最大任务数量
    let optimal_task_count = std::cmp::min(concurrency, 100);
    
    for _ in 0..optimal_task_count {
        let state = Arc::clone(test_state);
        let end_time = end_time;
        
        let task = tokio::spawn(async move {
            // 在测试时间内持续发送请求
            while std::time::Instant::now() < end_time {
                let request_start = std::time::Instant::now();
                
                match state.config.client.get(state.config.url.as_str()).send().await {
                    Ok(_response) => {
                        let latency = request_start.elapsed().as_millis() as u64;
                        state.stats.record_success(latency);
                    }
                    Err(_) => {
                        state.stats.record_failure();
                    }
                }
            }
        });
        
        tasks.push(task);
    }
    
    tasks
}

/// 辅助函数：等待任务完成
async fn wait_for_tasks(tasks: TaskList) {
    for task in tasks {
        task.await.unwrap();
    }
}

/// 辅助函数：生成测试结果
fn generate_test_result(
    test_state: &Arc<TestState>,
    start_time: std::time::Instant
) -> LoadTestResult {
    let duration = start_time.elapsed();
    let result = test_state.stats.get_results(duration);
    
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

    /// 简单测试：低并发，短时间，快速运行
    /// 用于开发调试，CI/CD自动运行
    #[tokio::test]
    async fn test_load_test_simple() {
        let config = Config {
            url: "http://localhost:8080/bench".to_string(),
            concurrency: 10,
            duration: 2, // 直接使用整数秒数
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
            duration: 10, // 直接使用整数秒数
        };
        
        let result = run(config).await;
        
        assert!(result.total_requests > 0);
        assert!(result.requests_per_second > 0.0);
    }
}
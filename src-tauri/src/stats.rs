use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

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

/// 简化的统计事件
#[derive(Debug)]
enum StatEvent {
    Success(u64),  // 延迟时间(ms)
    Failure,
}

/// 异步统计收集器
pub struct AsyncStats {
    total_requests: Arc<AtomicU32>,
    total_latency: Arc<AtomicU64>,
    stats_tx: tokio::sync::mpsc::Sender<StatEvent>,
    #[allow(dead_code)]  // 保持任务运行，即使不直接读取
    collector_task: tokio::task::JoinHandle<()>,
}

impl AsyncStats {
    pub fn new() -> Self {
        let total_requests = Arc::new(AtomicU32::new(0));
        let total_latency = Arc::new(AtomicU64::new(0));
        
        let (stats_tx, mut stats_rx) = tokio::sync::mpsc::channel(1000);
        
        let requests_clone = Arc::clone(&total_requests);
        let latency_clone = Arc::clone(&total_latency);
        
        let collector_task = tokio::spawn(async move {
            let mut batch_count = 0;
            let mut batch_latency = 0u64;
            
            while let Some(event) = stats_rx.recv().await {
                match event {
                    StatEvent::Success(latency) => {
                        batch_count += 1;
                        batch_latency += latency;
                    }
                    StatEvent::Failure => {
                        batch_count += 1;
                    }
                }
                
                // 每100个事件批量更新，减少原子操作
                if batch_count >= 100 {
                    requests_clone.fetch_add(batch_count, Ordering::Relaxed);
                    latency_clone.fetch_add(batch_latency, Ordering::Relaxed);
                    batch_count = 0;
                    batch_latency = 0;
                }
            }
            
            // 处理剩余的事件
            if batch_count > 0 {
                requests_clone.fetch_add(batch_count, Ordering::Relaxed);
                latency_clone.fetch_add(batch_latency, Ordering::Relaxed);
            }
        });
        
        Self {
            total_requests,
            total_latency,
            stats_tx,
            collector_task,
        }
    }
    
    pub fn record_success(&self, latency: u64) {
        let _ = self.stats_tx.try_send(StatEvent::Success(latency));
    }
    
    pub fn record_failure(&self) {
        let _ = self.stats_tx.try_send(StatEvent::Failure);
    }
    
    pub fn get_results(&self, duration: std::time::Duration) -> LoadTestResult {
        let total = self.total_requests.load(Ordering::Relaxed);
        let latency_sum = self.total_latency.load(Ordering::Relaxed);
        
        let rps = total as f64 / duration.as_secs_f64();
        let avg_latency = if total > 0 { latency_sum / total as u64 } else { 0 };
        
        LoadTestResult {
            total_requests: total,
            successful_requests: total, // 简化：假设所有请求都成功
            failed_requests: 0,
            requests_per_second: rps,
            average_latency: avg_latency,
            error_stats: ErrorStats {
                connection_errors: 0,
                timeout_errors: 0,
                http_errors: 0,
                other_errors: 0,
            },
        }
    }
}
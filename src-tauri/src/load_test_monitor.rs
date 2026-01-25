use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Duration;
use hdrhistogram::Histogram;
use sysinfo::System;
use tauri::Emitter;
use serde::{Deserialize, Serialize};
use crate::load_test;

/// 延迟分布统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyPercentiles {
    pub p50: u64, // 50%分位延迟（毫秒）
    pub p90: u64, // 90%分位延迟（毫秒）
    pub p95: u64, // 95%分位延迟（毫秒）
    pub p99: u64, // 99%分位延迟（毫秒）
}

/// 系统资源监控数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f64, // CPU使用率（%）
    pub memory_usage: f64, // 内存使用率（%）
    pub network_io: u64, // 网络IO（字节/秒）
}

/// 实时监控指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeMetrics {
    pub rps: f64, // 每秒请求数
    pub total_requests: u32, // 总请求数
    pub successful_requests: u32, // 成功请求数
    pub failed_requests: u32, // 失败请求数
    pub average_latency: u64, // 平均延迟（毫秒）
    pub latency_percentiles: LatencyPercentiles, // 延迟分布
    pub system_metrics: SystemMetrics, // 系统资源使用情况
}

/// 监控装饰器 - 通过回调机制增强负载测试功能
pub struct LoadTestMonitor {
    monitor: Arc<Monitor>,
    app_handle: Option<tauri::AppHandle>,
}

impl LoadTestMonitor {
    /// 创建新的监控装饰器
    pub fn new() -> Self {
        Self {
            monitor: Arc::new(Monitor::new()),
            app_handle: None,
        }
    }

    /// 设置Tauri应用句柄，用于实时事件推送
    pub fn with_app_handle(mut self, app_handle: tauri::AppHandle) -> Self {
        self.app_handle = Some(app_handle);
        self
    }

    /// 装饰负载测试函数，添加监控功能
    pub async fn run_with_monitoring(&self, config: load_test::Config) -> load_test::LoadTestResult {
        let start_time = std::time::Instant::now();
        let end_time = start_time + Duration::from_secs(config.duration);
        
        // 启动实时监控推送任务
        let monitor_clone = Arc::clone(&self.monitor);
        let app_handle_clone = self.app_handle.clone();
        
        let monitoring_task = if let Some(app_handle) = app_handle_clone {
            Some(tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_millis(500));
                while std::time::Instant::now() < end_time {
                    interval.tick().await;
                    let metrics = monitor_clone.collect_metrics();
                    let _ = app_handle.emit("load_test_metrics", metrics);
                }
            }))
        } else {
            None
        };

        // 执行原始负载测试
        let result = load_test::run(config).await;

        // 等待监控任务完成
        if let Some(task) = monitoring_task {
            task.await.unwrap();
        }

        // 打印监控数据
        let metrics = self.monitor.collect_metrics();
        println!("\n=== 监控数据 ===");
        println!("每秒请求数: {:.2} RPS", metrics.rps);
        println!("延迟分布:");
        println!("  P50: {}ms", metrics.latency_percentiles.p50);
        println!("  P90: {}ms", metrics.latency_percentiles.p90);
        println!("  P95: {}ms", metrics.latency_percentiles.p95);
        println!("  P99: {}ms", metrics.latency_percentiles.p99);
        println!("系统资源:");
        println!("  CPU使用率: {:.1}%", metrics.system_metrics.cpu_usage);
        println!("  内存使用率: {:.1}%", metrics.system_metrics.memory_usage);
        println!("  网络IO: {} bytes/s", metrics.system_metrics.network_io);

        result
    }


}

/// 内部监控器实现
struct Monitor {
    latency_histogram: Arc<std::sync::Mutex<Histogram<u64>>>,
    total_requests: Arc<AtomicU32>,
    successful_requests: Arc<AtomicU32>,
    failed_requests: Arc<AtomicU32>,
    total_latency: Arc<AtomicU64>,
    start_time: std::time::Instant,
    system: Arc<std::sync::Mutex<System>>,
}

impl Monitor {
    fn new() -> Self {
        Self {
            latency_histogram: Arc::new(std::sync::Mutex::new(Histogram::new(3).unwrap())),
            total_requests: Arc::new(AtomicU32::new(0)),
            successful_requests: Arc::new(AtomicU32::new(0)),
            failed_requests: Arc::new(AtomicU32::new(0)),
            total_latency: Arc::new(AtomicU64::new(0)),
            start_time: std::time::Instant::now(),
            system: Arc::new(std::sync::Mutex::new(System::new_all())),
        }
    }



    fn collect_metrics(&self) -> RealTimeMetrics {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let total_requests = self.total_requests.load(Ordering::Relaxed);
        let successful_requests = self.successful_requests.load(Ordering::Relaxed);
        let failed_requests = self.failed_requests.load(Ordering::Relaxed);
        let total_latency = self.total_latency.load(Ordering::Relaxed);
        
        // 计算RPS
        let rps = if elapsed > 0.0 {
            total_requests as f64 / elapsed
        } else {
            0.0
        };
        
        // 计算平均延迟
        let average_latency = if successful_requests > 0 {
            total_latency / successful_requests as u64
        } else {
            0
        };
        
        // 计算延迟分布
        let histogram = self.latency_histogram.lock().unwrap();
        let latency_percentiles = LatencyPercentiles {
            p50: histogram.value_at_percentile(50.0),
            p90: histogram.value_at_percentile(90.0),
            p95: histogram.value_at_percentile(95.0),
            p99: histogram.value_at_percentile(99.0),
        };
        
        // 系统资源监控
        let mut system = self.system.lock().unwrap();
        system.refresh_all();
        
        let cpu_usage = system.global_cpu_usage() as f64;
        let total_memory = system.total_memory() as f64;
        let used_memory = system.used_memory() as f64;
        let memory_usage = (used_memory / total_memory) * 100.0;
        
        // 网络IO监控（简化实现）
        let network_io = 0u64;
        
        let system_metrics = SystemMetrics {
            cpu_usage,
            memory_usage,
            network_io,
        };
        
        RealTimeMetrics {
            rps,
            total_requests,
            successful_requests,
            failed_requests,
            average_latency,
            latency_percentiles,
            system_metrics,
        }
    }
}
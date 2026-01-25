use hdrhistogram::Histogram;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Duration;

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

/// 监控器结构体
pub struct Monitor {
    // 延迟分布统计
    latency_histogram: Arc<Mutex<Histogram<u64>>>,
    
    // 基本统计数据
    total_requests: Arc<AtomicU32>,
    successful_requests: Arc<AtomicU32>,
    failed_requests: Arc<AtomicU32>,
    total_latency: Arc<AtomicU64>,
    
    // 启动时间
    start_time: std::time::Instant,
}

impl Monitor {
    /// 创建新的监控器
    pub fn new() -> Self {
        Self {
            latency_histogram: Arc::new(Mutex::new(Histogram::new(3).unwrap())),
            total_requests: Arc::new(AtomicU32::new(0)),
            successful_requests: Arc::new(AtomicU32::new(0)),
            failed_requests: Arc::new(AtomicU32::new(0)),
            total_latency: Arc::new(AtomicU64::new(0)),
            start_time: std::time::Instant::now(),
        }
    }

    /// 记录成功请求
    pub fn record_success(&self, latency: Duration) {
        let latency_ms = latency.as_millis() as u64;
        
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
        self.total_latency.fetch_add(latency_ms, Ordering::Relaxed);
        
        // 记录延迟分布
        let mut histogram = self.latency_histogram.lock().unwrap();
        histogram.record(latency_ms).unwrap();
    }

    /// 记录失败请求
    pub fn record_failure(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// 收集实时指标
    pub fn collect_metrics(&self) -> RealTimeMetrics {
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
        
        // 系统资源监控（简化实现，实际项目可集成sysinfo库）
        let system_metrics = SystemMetrics {
            cpu_usage: 0.0, // 占位值，实际可通过sysinfo库获取
            memory_usage: 0.0, // 占位值，实际可通过sysinfo库获取
            network_io: 0, // 占位值，实际可通过sysinfo库获取
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

<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  // 负载测试相关
  let url = $state("http://httpbin.org/get");
  let concurrency = $state(100); // 并发数，默认100
  let duration = $state(10); // 测试时长，默认10秒
  let enableMonitoring = $state(true); // 是否启用监控
  let testResult = $state<any>(null);
  let isLoading = $state(false);
  let realTimeMetrics = $state<any>(null);
  
  // 监听Tauri事件
  import { listen } from '@tauri-apps/api/event';
  
  // 历史数据存储
  let cpuHistory = $state<number[]>([]);
  let memoryHistory = $state<number[]>([]);
  const maxHistoryPoints = 50; // 最多显示50个数据点
  
  // 初始化时监听监控事件
  listen('load_test_metrics', (event) => {
    realTimeMetrics = event.payload;
    
    // 更新历史数据
    if (realTimeMetrics?.system_metrics) {
      cpuHistory.push(realTimeMetrics.system_metrics.cpu_usage);
      memoryHistory.push(realTimeMetrics.system_metrics.memory_usage);
      
      // 限制历史数据长度
      if (cpuHistory.length > maxHistoryPoints) {
        cpuHistory = cpuHistory.slice(-maxHistoryPoints);
        memoryHistory = memoryHistory.slice(-maxHistoryPoints);
      }
      
      // 更新图表
      updateCharts();
    }
  });
  
  // 更新图表函数
  function updateCharts() {
    // 简单的Canvas图表绘制
    drawChart('cpu-chart', cpuHistory, '#ff6b6b');
    drawChart('memory-chart', memoryHistory, '#4ecdc4');
  }
  
  // 绘制简单折线图
  function drawChart(canvasId: string, data: number[], color: string) {
    const canvas = document.getElementById(canvasId) as HTMLCanvasElement;
    if (!canvas) return;
    
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    
    // 设置Canvas尺寸
    canvas.width = canvas.offsetWidth;
    canvas.height = canvas.offsetHeight;
    
    const width = canvas.width;
    const height = canvas.height;
    
    // 清空画布
    ctx.clearRect(0, 0, width, height);
    
    if (data.length < 2) return;
    
    // 绘制网格线
    ctx.strokeStyle = '#e0e0e0';
    ctx.lineWidth = 0.5;
    for (let i = 0; i <= 4; i++) {
      const y = height * i / 4;
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }
    
    // 绘制折线
    ctx.strokeStyle = color;
    ctx.lineWidth = 2;
    ctx.beginPath();
    
    data.forEach((value, index) => {
      const x = (index / (data.length - 1)) * width;
      const y = height - (value / 100) * height;
      
      if (index === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    });
    
    ctx.stroke();
    
    // 绘制数据点
    ctx.fillStyle = color;
    data.forEach((value, index) => {
      const x = (index / (data.length - 1)) * width;
      const y = height - (value / 100) * height;
      
      ctx.beginPath();
      ctx.arc(x, y, 2, 0, 2 * Math.PI);
      ctx.fill();
    });
  }
  

  
  async function runLoadTest(event: Event) {
    event.preventDefault();
    isLoading = true;
    testResult = null;
    realTimeMetrics = null;
    
    try {
      if (enableMonitoring) {
        // 调用带监控的负载测试命令
        testResult = await invoke("run_load_test_with_monitoring", {
          config: {
            url,
            concurrency,
            duration
          }
        });
      } else {
        // 调用原始负载测试命令
        testResult = await invoke("run_load_test", {
          config: {
            url,
            concurrency,
            duration
          }
        });
      }
    } catch (error) {
      console.error("负载测试失败:", error);
      testResult = { error: String(error) };
    } finally {
      isLoading = false;
    }
  }
</script>

<main class="container">
  <h1>高性能负载测试工具</h1>

  <!-- 负载测试部分 -->
  <form class="load-test-form" onsubmit={runLoadTest}>
    <div class="row">
      <input 
        id="load-test-url" 
        placeholder="Enter URL to test..." 
        bind:value={url} 
        style="margin-right: 5px; flex: 1;" 
      />
    </div>
    
    <div class="row" style="margin-top: 10px; align-items: center;">
      <label for="concurrency" style="margin-right: 5px; font-weight: bold;">并发数:</label>
      <input 
        id="concurrency"
        type="number" 
        placeholder="100" 
        bind:value={concurrency} 
        style="margin-right: 15px; width: 120px;" 
        min="1" 
      />
      <label for="duration" style="margin-right: 5px; font-weight: bold;">测试时长 (秒):</label>
      <input 
        id="duration"
        type="number" 
        placeholder="10" 
        bind:value={duration} 
        style="margin-right: 15px; width: 120px;" 
        min="1" 
      />
      <label for="monitoring" style="margin-right: 5px; font-weight: bold;">启用监控:</label>
      <input 
        id="monitoring"
        type="checkbox" 
        bind:checked={enableMonitoring} 
        style="margin-right: 15px;" 
      />
      <button type="submit" disabled={isLoading}>
        {isLoading ? "测试中..." : "执行负载测试"}
      </button>
    </div>
  </form>
  
  <!-- 实时监控数据 -->
  {#if realTimeMetrics && enableMonitoring}
    <div class="real-time-metrics">
      <h3>实时监控数据</h3>
      
      <!-- 性能指标 -->
      <div class="performance-metrics">
        <div class="metric-row">
          <div class="metric-card">
            <strong>RPS:</strong> {realTimeMetrics.rps.toFixed(2)}
          </div>
          <div class="metric-card">
            <strong>总请求数:</strong> {realTimeMetrics.total_requests}
          </div>
          <div class="metric-card">
            <strong>成功请求:</strong> {realTimeMetrics.successful_requests}
          </div>
          <div class="metric-card">
            <strong>失败请求:</strong> {realTimeMetrics.failed_requests}
          </div>
        </div>
      </div>
      
      <!-- 系统资源监控曲线 -->
      <div class="resource-charts">
        <div class="chart-container">
          <h4>CPU使用率 (%)</h4>
          <div class="chart">
            <div class="chart-bar" style="width: {Math.min(realTimeMetrics.system_metrics.cpu_usage, 100)}%"></div>
            <span class="chart-value">{realTimeMetrics.system_metrics.cpu_usage.toFixed(1)}%</span>
          </div>
        </div>
        
        <div class="chart-container">
          <h4>内存使用率 (%)</h4>
          <div class="chart">
            <div class="chart-bar" style="width: {Math.min(realTimeMetrics.system_metrics.memory_usage, 100)}%"></div>
            <span class="chart-value">{realTimeMetrics.system_metrics.memory_usage.toFixed(1)}%</span>
          </div>
        </div>
      </div>
      
      <!-- 历史数据曲线 -->
      <div class="history-charts">
        <div class="history-container">
          <h4>CPU使用率趋势</h4>
          <canvas class="history-chart" id="cpu-chart"></canvas>
        </div>
        
        <div class="history-container">
          <h4>内存使用率趋势</h4>
          <canvas class="history-chart" id="memory-chart"></canvas>
        </div>
      </div>
    </div>
  {/if}
  
  {#if testResult}
    {#if testResult.error}
      <div class="error-result">
        <h3>测试失败</h3>
        <p>{testResult.error}</p>
      </div>
    {:else}
      <div class="test-result">
        <h3>测试结果</h3>
        <div class="result-grid">
          <div class="result-item">
            <strong>总请求数:</strong> {testResult.total_requests}
          </div>
          <div class="result-item">
            <strong>成功请求数:</strong> {testResult.successful_requests}
          </div>
          <div class="result-item">
            <strong>失败请求数:</strong> {testResult.failed_requests}
          </div>
          <div class="result-item">
            <strong>每秒请求数:</strong> {testResult.requests_per_second.toFixed(2)}
          </div>
          <div class="result-item">
            <strong>平均延迟:</strong> {testResult.average_latency}ms
          </div>
        </div>
        
        <h4>错误统计</h4>
        <div class="result-grid">
          <div class="result-item">
            <strong>连接错误:</strong> {testResult.error_stats.connection_errors}
          </div>
          <div class="result-item">
            <strong>超时错误:</strong> {testResult.error_stats.timeout_errors}
          </div>
          <div class="result-item">
            <strong>HTTP错误:</strong> {testResult.error_stats.http_errors}
          </div>
          <div class="result-item">
            <strong>其他错误:</strong> {testResult.error_stats.other_errors}
          </div>
        </div>
      </div>
    {/if}
  {/if}
</main>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
  max-width: 800px;
  margin: 0 auto;
  padding: 0 20px;
}

.row {
  display: flex;
  justify-content: center;
  margin: 1em 0;
  gap: 5px;
}

h1 {
  text-align: center;
}

h2 {
  margin-top: 2em;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

input,
button {
  outline: none;
}

/* 测试结果样式 */
.test-result,
.error-result {
  margin-top: 1.5em;
  padding: 1.5em;
  border-radius: 8px;
  text-align: left;
  background-color: #f0f0f0;
}

.error-result {
  background-color: #ffebee;
  border: 1px solid #ffcdd2;
}

.result-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1em;
  margin: 1em 0;
}

.result-item {
  padding: 0.8em;
  background-color: rgba(255, 255, 255, 0.8);
  border-radius: 6px;
}

/* 实时监控数据样式 */
.real-time-metrics {
  margin-top: 1.5em;
  padding: 1.5em;
  border-radius: 8px;
  text-align: left;
  background-color: #e8f5e8;
  border: 1px solid #c8e6c9;
}

/* 性能指标样式 */
.performance-metrics {
  margin: 1em 0;
}

.metric-row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 1em;
}

.metric-card {
  padding: 1em;
  background-color: rgba(255, 255, 255, 0.9);
  border-radius: 8px;
  text-align: center;
  border: 1px solid #c8e6c9;
}

.metric-card strong {
  display: block;
  margin-bottom: 0.5em;
  color: #2e7d32;
}

/* 资源图表样式 */
.resource-charts {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 1.5em;
  margin: 1.5em 0;
}

.chart-container {
  background-color: rgba(255, 255, 255, 0.9);
  padding: 1em;
  border-radius: 8px;
  border: 1px solid #c8e6c9;
}

.chart-container h4 {
  margin: 0 0 1em 0;
  color: #2e7d32;
}

.chart {
  position: relative;
  height: 40px;
  background-color: #f5f5f5;
  border-radius: 4px;
  overflow: hidden;
}

.chart-bar {
  height: 100%;
  background: linear-gradient(90deg, #4caf50, #8bc34a);
  transition: width 0.3s ease;
}

.chart-value {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-weight: bold;
  color: #2e7d32;
  text-shadow: 0 0 2px white;
}

/* 历史曲线样式 */
.history-charts {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1.5em;
  margin: 1.5em 0;
}

.history-container {
  background-color: rgba(255, 255, 255, 0.9);
  padding: 1em;
  border-radius: 8px;
  border: 1px solid #c8e6c9;
}

.history-container h4 {
  margin: 0 0 1em 0;
  color: #2e7d32;
}

.history-chart {
  width: 100%;
  height: 120px;
  background-color: #f8f9fa;
  border-radius: 4px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
  
  .test-result {
    background-color: #3a3a3a;
  }
  
  .error-result {
    background-color: #4a2c2c;
    border-color: #6a4a4a;
  }
  
  .result-item {
    background-color: rgba(0, 0, 0, 0.3);
  }
}
</style>
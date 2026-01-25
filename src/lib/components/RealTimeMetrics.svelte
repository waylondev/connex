<script lang="ts">
  import Charts from './Charts.svelte';
  
  export let realTimeMetrics: any;
  export let cpuHistory: number[] = [];
  export let memoryHistory: number[] = [];
</script>

<!-- 实时监控数据 -->
{#if realTimeMetrics}
  <div class="real-time-metrics">
    <h3>实时监控数据</h3>
    
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
    <Charts {cpuHistory} {memoryHistory} />
  </div>
{/if}

<style>
.real-time-metrics {
  margin: 2rem 0;
  padding: 1rem;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  background-color: #f9f9f9;
}

.resource-charts {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
  margin-bottom: 1.5rem;
}

.chart-container {
  background: white;
  padding: 0.8rem;
  border-radius: 6px;
  border: 1px solid #ddd;
}

.chart-container h4 {
  margin: 0 0 0.5rem 0;
  font-size: 0.9rem;
  color: #333;
}

.chart {
  position: relative;
  height: 30px;
  background: #f0f0f0;
  border-radius: 4px;
  overflow: hidden;
}

.chart-bar {
  height: 100%;
  background: linear-gradient(90deg, #4ecdc4, #44a08d);
  transition: width 0.3s ease;
}

.chart-value {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: 0.8rem;
  font-weight: bold;
  color: #333;
}
</style>
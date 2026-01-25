<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from '@tauri-apps/api/event';
  
  // 导入组件
  import LoadTestConfig from '$lib/components/LoadTestConfig.svelte';
  import RealTimeMetrics from '$lib/components/RealTimeMetrics.svelte';
  import TestResults from '$lib/components/TestResults.svelte';

  // 负载测试状态
  let url = $state("http://httpbin.org/get");
  let concurrency = $state(100);
  let duration = $state(10);
  let enableMonitoring = $state(true);
  let testResult = $state<any>(null);
  let isLoading = $state(false);
  let realTimeMetrics = $state<any>(null);
  
  // 历史数据存储
  let cpuHistory = $state<number[]>([]);
  let memoryHistory = $state<number[]>([]);
  const maxHistoryPoints = 50;
  
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
    }
  });
  
  // 执行负载测试
  async function runLoadTest(event: Event) {
    event.preventDefault();
    isLoading = true;
    testResult = null;
    realTimeMetrics = null;
    cpuHistory = [];
    memoryHistory = [];
    
    try {
      const config = {
        url,
        concurrency,
        duration
      };
      
      if (enableMonitoring) {
        // 调用带监控的负载测试命令
        testResult = await invoke("run_load_test_with_monitoring", { config });
      } else {
        // 调用原始负载测试命令
        testResult = await invoke("run_load_test", { config });
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

  <!-- 负载测试配置 -->
  <LoadTestConfig 
    {url} {concurrency} {duration} {enableMonitoring} {isLoading}
    onRunTest={runLoadTest}
  />
  
  <!-- 实时监控数据 -->
  {#if enableMonitoring}
    <RealTimeMetrics 
      {realTimeMetrics}
      {cpuHistory}
      {memoryHistory}
    />
  {/if}
  
  <!-- 测试结果 -->
  <TestResults {testResult} />
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
  padding-top: 5vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
  max-width: 900px;
  margin: 0 auto;
  padding: 0 20px;
}

h1 {
  text-align: center;
  margin-bottom: 2rem;
  color: #333;
}
</style>
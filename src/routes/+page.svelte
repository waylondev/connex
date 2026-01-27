<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  
  // 导入组件
  import LoadTestConfig from '$lib/components/LoadTestConfig.svelte';
  import TestResults from '$lib/components/TestResults.svelte';

  // 负载测试状态
  let url = $state("http://localhost:3000");
  let concurrency = $state(1000);
  let duration = $state(10);
  let testResult = $state<any>(null);
  let isLoading = $state(false);

  
  // 执行负载测试
  async function runLoadTest(event: Event) {
    event.preventDefault();
    isLoading = true;
    testResult = null;
    
    // 调试：打印实际传递的参数
    console.log('前端传递的参数:', {
      url,
      concurrency,
    duration
  });
    
    try {
      const config = {
        url,
        concurrency,
        duration
      };
      
      // 调用负载测试命令
      testResult = await invoke("run_load_test", { config });
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
    bind:url
    bind:concurrency
    bind:duration
    {isLoading}
    onRunTest={runLoadTest}
  />
  
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
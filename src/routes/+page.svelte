<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  // 负载测试相关
  let url = $state("https://api.example.com");
  let testResult = $state<any>(null);
  let isLoading = $state(false);
  
  async function runLoadTest(event: Event) {
    event.preventDefault();
    isLoading = true;
    testResult = null;
    
    try {
      // 调用后端负载测试命令，传递完整的配置对象
      testResult = await invoke("run_load_test", {
        config: {
          url
        }
      });
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
  <form class="row" onsubmit={runLoadTest}>
    <input 
      id="load-test-url" 
      placeholder="Enter URL to test..." 
      bind:value={url} 
      style="margin-right: 5px; flex: 1;" 
    />
    <button type="submit" disabled={isLoading}>
      {isLoading ? "测试中..." : "执行负载测试"}
    </button>
  </form>
  
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
            <strong>平均延迟:</strong> {(testResult.average_latency.as_millis() || 0)}ms
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
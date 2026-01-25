<script lang="ts">
  export let testResult: any = null;
</script>

<!-- 测试结果展示 -->
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

<style>
.test-result {
  margin: 2rem 0;
  padding: 1.5rem;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  background-color: #f9f9f9;
}

.error-result {
  margin: 2rem 0;
  padding: 1.5rem;
  border: 1px solid #f44336;
  border-radius: 8px;
  background-color: #ffebee;
  color: #c62828;
}

.result-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
  margin: 1rem 0;
}

.result-item {
  padding: 0.8rem;
  background: white;
  border-radius: 6px;
  border: 1px solid #ddd;
}

.result-item strong {
  color: #333;
  margin-right: 0.5rem;
}

h3, h4 {
  margin: 0 0 0.5rem 0;
  color: #333;
}

h4 {
  margin-top: 1.5rem;
  font-size: 0.9rem;
}
</style>
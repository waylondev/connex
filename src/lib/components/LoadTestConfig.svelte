<script lang="ts">
  // 使用响应式绑定，确保双向数据流
  export let url: string;
  export let concurrency: number;
  export let duration: number;
  export let enableMonitoring: boolean;
  export let isLoading: boolean;
  
  // 事件定义
  export let onRunTest: (event: Event) => void;
  
  // 确保数据变化时通知父组件
  $: {
    // 当这些值变化时，确保父组件能感知到
    url, concurrency, duration, enableMonitoring;
  }
</script>

<!-- 负载测试配置表单 -->
<form class="load-test-form" onsubmit={onRunTest}>
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

<style>
.load-test-form {
  margin-bottom: 2rem;
}

.row {
  display: flex;
  justify-content: center;
  margin: 0.5em 0;
  gap: 0.5rem;
  align-items: center;
}

input {
  border-radius: 8px;
  border: 1px solid #ccc;
  padding: 0.6em 1.2em;
  font-size: 0.9em;
  background-color: #ffffff;
}

input[type="number"] {
  width: 120px;
}

button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 0.9em;
  font-weight: 500;
  background-color: #646cff;
  color: white;
  cursor: pointer;
}

button:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

button:hover:not(:disabled) {
  background-color: #535bf2;
}
</style>
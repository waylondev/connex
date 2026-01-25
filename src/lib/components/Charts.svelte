<script lang="ts">
  import { onMount, afterUpdate } from 'svelte';
  
  export let cpuHistory: number[] = [];
  export let memoryHistory: number[] = [];
  
  let cpuCanvas: HTMLCanvasElement;
  let memoryCanvas: HTMLCanvasElement;
  
  // 计算统计数据
  function calculateStats(data: number[]) {
    if (data.length === 0) return { max: 0, min: 0, avg: 0 };
    
    const max = Math.max(...data);
    const min = Math.min(...data);
    const avg = data.reduce((sum, val) => sum + val, 0) / data.length;
    
    return { max, min, avg };
  }
  
  // 绘制简单折线图
  function drawChart(canvas: HTMLCanvasElement, data: number[], color: string) {
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
  
  // 更新图表
  function updateCharts() {
    if (cpuCanvas) drawChart(cpuCanvas, cpuHistory, '#ff6b6b');
    if (memoryCanvas) drawChart(memoryCanvas, memoryHistory, '#4ecdc4');
  }
  
  onMount(() => {
    updateCharts();
  });
  
  afterUpdate(() => {
    updateCharts();
  });
</script>

<!-- 历史数据曲线 -->
<div class="history-charts">
  <div class="history-container">
    <h4>CPU使用率趋势</h4>
    <canvas class="history-chart" bind:this={cpuCanvas}></canvas>
    {#if cpuHistory.length > 0}
      <div class="stats">
        <span class="stat-item">最大值: {calculateStats(cpuHistory).max.toFixed(1)}%</span>
        <span class="stat-item">最小值: {calculateStats(cpuHistory).min.toFixed(1)}%</span>
        <span class="stat-item">平均值: {calculateStats(cpuHistory).avg.toFixed(1)}%</span>
      </div>
    {/if}
  </div>
  
  <div class="history-container">
    <h4>内存使用率趋势</h4>
    <canvas class="history-chart" bind:this={memoryCanvas}></canvas>
    {#if memoryHistory.length > 0}
      <div class="stats">
        <span class="stat-item">最大值: {calculateStats(memoryHistory).max.toFixed(1)}%</span>
        <span class="stat-item">最小值: {calculateStats(memoryHistory).min.toFixed(1)}%</span>
        <span class="stat-item">平均值: {calculateStats(memoryHistory).avg.toFixed(1)}%</span>
      </div>
    {/if}
  </div>
</div>

<style>
.history-charts {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
}

.history-container {
  background: white;
  padding: 0.8rem;
  border-radius: 6px;
  border: 1px solid #ddd;
}

.history-container h4 {
  margin: 0 0 0.5rem 0;
  font-size: 0.9rem;
  color: #333;
}

.history-chart {
  width: 100%;
  height: 120px;
  border-radius: 4px;
  background: #f8f9fa;
}

.stats {
  display: flex;
  justify-content: space-between;
  margin-top: 0.5rem;
  font-size: 0.7rem;
  color: #666;
}

.stat-item {
  padding: 0.2rem 0.4rem;
  background: #f0f0f0;
  border-radius: 3px;
}
</style>
<script lang="ts">
  interface Props {
    data: Array<{ timestamp: number; value: number }>;
    label: string;
    unit?: string;
    color?: string;
    maxPoints?: number;
    minValue?: number;
    maxValue?: number;
    height?: number;
  }

  let {
    data = [],
    label,
    unit = '%',
    color = 'var(--cpu-color)',
    maxPoints = 150,
    minValue = 0,
    maxValue = 100,
    height = 120
  }: Props = $props();

  let width = $state(0);
  let mouseX = $state<number | null>(null);

  let latestTime = $derived(data.length > 0 ? data[data.length - 1].timestamp : Date.now());
  let startTime = $derived(latestTime - (maxPoints * 2000));

  let currentValue = $derived(data.length > 0 ? data[data.length - 1].value : 0);

  let pathData = $derived.by(() => {
    if (data.length === 0 || width === 0) return '';
    const points = data.map(d => {
      const x = ((d.timestamp - startTime) / (latestTime - startTime)) * width;
      const y = height - ((d.value - minValue) / (maxValue - minValue)) * height;
      return `${x},${y}`;
    });
    return `M ${points.join(' L ')}`;
  });

  let fillPathData = $derived.by(() => {
    if (!pathData || data.length === 0 || width === 0) return '';
    const firstX = ((data[0].timestamp - startTime) / (latestTime - startTime)) * width;
    const lastX = ((data[data.length - 1].timestamp - startTime) / (latestTime - startTime)) * width;
    return `${pathData} L ${lastX},${height} L ${firstX},${height} Z`;
  });

  let hoveredPoint = $derived.by(() => {
    if (mouseX === null || width === 0 || data.length === 0) return null;
    const timeAtMouse = startTime + (mouseX / width) * (latestTime - startTime);
    
    let closest = data[0];
    let minDiff = Math.abs(data[0].timestamp - timeAtMouse);
    
    for (let i = 1; i < data.length; i++) {
      const diff = Math.abs(data[i].timestamp - timeAtMouse);
      if (diff < minDiff) {
        minDiff = diff;
        closest = data[i];
      }
    }
    return closest;
  });

  const timeLabels = [
    { label: '5m', pct: 0 },
    { label: '4m', pct: 0.2 },
    { label: '3m', pct: 0.4 },
    { label: '2m', pct: 0.6 },
    { label: '1m', pct: 0.8 },
    { label: 'now', pct: 1.0 }
  ];

  let gradientId = $derived(`gradient-${label.toLowerCase().replace(/[^a-z0-9]/g, '-')}-${Math.random().toString(36).slice(2, 7)}`);
</script>

<div class="chart-wrapper">
  <div class="header">
    <div class="label-container">
      <span class="label-text">{label}</span>
    </div>
    <div class="value-container">
      <span class="value-text" style="color: {color}">{currentValue.toFixed(1)}{unit}</span>
    </div>
  </div>

  <div class="svg-container" bind:clientWidth={width}>
    <svg 
      {width} 
      height={height + 20} 
      class="chart"
      onmousemove={(e) => mouseX = e.offsetX}
      onmouseleave={() => mouseX = null}
      role="img"
      aria-label={label}
    >
      <defs>
        <linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stop-color={color} stop-opacity="0.3" />
          <stop offset="100%" stop-color={color} stop-opacity="0.0" />
        </linearGradient>
      </defs>

      {#if width > 0}
        {#each [0, 0.25, 0.5, 0.75, 1] as pct}
          {@const y = height - pct * height}
          {@const val = minValue + pct * (maxValue - minValue)}
          <line 
            x1="0" 
            y1={y} 
            x2={width} 
            y2={y} 
            stroke="var(--border-color)" 
            stroke-dasharray="4, 4" 
            stroke-width="1"
            class="grid-line"
          />
          {#if pct > 0}
            <text 
              x="0" 
              y={y > 10 ? y - 4 : y + 12} 
              class="axis-text"
            >
              {val}{unit}
            </text>
          {/if}
        {/each}

        {#each timeLabels as timeLabel}
          {@const x = timeLabel.pct * width}
          <text 
            x={timeLabel.pct === 0 ? 0 : timeLabel.pct === 1 ? width : x} 
            y={height + 16} 
            class="axis-text time-text"
            text-anchor={timeLabel.pct === 0 ? "start" : timeLabel.pct === 1 ? "end" : "middle"}
          >
            {timeLabel.label}
          </text>
        {/each}

        {#if pathData}
          <path 
            d={fillPathData} 
            fill="url(#{gradientId})" 
            stroke="none"
          />
          <path 
            d={pathData} 
            fill="none" 
            stroke={color} 
            stroke-width="2" 
            stroke-linejoin="round"
            stroke-linecap="round"
          />
        {/if}

        {#if hoveredPoint}
          {@const hx = ((hoveredPoint.timestamp - startTime) / (latestTime - startTime)) * width}
          {@const hy = height - ((hoveredPoint.value - minValue) / (maxValue - minValue)) * height}
          
          <line 
            x1={hx} 
            y1="0" 
            x2={hx} 
            y2={height} 
            stroke="var(--text-secondary)" 
            stroke-width="1"
            class="crosshair"
          />
          
          <circle 
            cx={hx} 
            cy={hy} 
            r="4" 
            fill={color} 
            stroke="var(--bg-primary)"
            stroke-width="2"
          />
          
          {@const tooltipWidth = 80}
          {@const tooltipHeight = 40}
          {@const tx = hx > width - tooltipWidth - 10 ? hx - tooltipWidth - 10 : hx + 10}
          {@const ty = hy < tooltipHeight + 10 ? hy + 10 : hy - tooltipHeight - 10}
          
          <g transform="translate({tx}, {ty})" class="tooltip">
            <rect 
              x="0" 
              y="0" 
              width={tooltipWidth} 
              height={tooltipHeight} 
              rx="4" 
              fill="var(--bg-surface-hover)"
              stroke="var(--border-color)"
              stroke-width="1"
            />
            <text 
              x="6" 
              y="16" 
              class="tooltip-time"
            >
              {new Date(hoveredPoint.timestamp).toLocaleTimeString([], {hour12: false, hour: '2-digit', minute:'2-digit', second:'2-digit'})}
            </text>
            <text 
              x="6" 
              y="32" 
              class="tooltip-value"
              fill={color}
            >
              {hoveredPoint.value.toFixed(1)}{unit}
            </text>
          </g>
        {/if}
      {/if}
    </svg>
  </div>
</div>

<style>
  .chart-wrapper {
    display: flex;
    flex-direction: column;
    width: 100%;
    background-color: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    padding: var(--space-4);
    box-sizing: border-box;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-2);
  }

  .label-text {
    font-family: var(--font-mono);
    font-size: var(--font-sm);
    color: var(--text-secondary);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .value-text {
    font-family: var(--font-mono);
    font-size: var(--font-lg);
    font-weight: 700;
  }

  .svg-container {
    width: 100%;
    position: relative;
    cursor: crosshair;
  }

  .chart {
    display: block;
    overflow: visible;
  }

  .grid-line {
    opacity: 0.5;
  }

  .axis-text {
    font-family: var(--font-mono);
    font-size: 10px;
    fill: var(--text-muted);
    user-select: none;
  }

  .time-text {
    fill: var(--text-secondary);
  }

  .tooltip-time {
    font-family: var(--font-mono);
    font-size: 10px;
    fill: var(--text-secondary);
    user-select: none;
  }

  .tooltip-value {
    font-family: var(--font-mono);
    font-size: var(--font-sm);
    font-weight: 700;
    user-select: none;
  }

  .crosshair {
    opacity: 0.7;
    stroke-dasharray: 4, 4;
  }
</style>

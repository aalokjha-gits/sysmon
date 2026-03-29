<script lang="ts">
  interface Props {
    coreData: Map<number, Array<{ timestamp: number; value: number }>>;
    height?: number;
    maxPoints?: number;
  }

  let {
    coreData,
    height = 160,
    maxPoints = 150
  }: Props = $props();

  const colors = [
    '#3b82f6', '#22c55e', '#f59e0b', '#ef4444', 
    '#8b5cf6', '#06b6d4', '#ec4899', '#f97316', 
    '#14b8a6', '#6366f1', '#84cc16', '#e879f9'
  ];

  let width = $state(0);
  let mouseX = $state<number | null>(null);

  let coresArray = $derived(Array.from(coreData.entries()).sort((a, b) => a[0] - b[0]));

  let latestTime = $derived.by(() => {
    let latest = 0;
    for (const data of coreData.values()) {
      if (data.length > 0 && data[data.length - 1].timestamp > latest) {
        latest = data[data.length - 1].timestamp;
      }
    }
    return latest || Date.now();
  });

  let startTime = $derived(latestTime - (maxPoints * 2000));

  let overallCurrentValue = $derived.by(() => {
    if (coresArray.length === 0) return 0;
    let sum = 0;
    let count = 0;
    for (const [_, data] of coresArray) {
      if (data.length > 0) {
        sum += data[data.length - 1].value;
        count++;
      }
    }
    return count > 0 ? sum / count : 0;
  });

  let hoveredTime = $derived.by(() => {
    if (mouseX === null || width === 0) return null;
    return startTime + (mouseX / width) * (latestTime - startTime);
  });

  let legendValues = $derived.by(() => {
    const values = new Map<number, number>();
    for (const [core, data] of coresArray) {
      if (data.length === 0) {
        values.set(core, 0);
        continue;
      }
      if (hoveredTime !== null) {
        let closest = data[0];
        let minDiff = Math.abs(data[0].timestamp - hoveredTime);
        for (let i = 1; i < data.length; i++) {
          const diff = Math.abs(data[i].timestamp - hoveredTime);
          if (diff < minDiff) {
            minDiff = diff;
            closest = data[i];
          }
        }
        values.set(core, closest.value);
      } else {
        values.set(core, data[data.length - 1].value);
      }
    }
    return values;
  });

  let hoverX = $derived.by(() => {
    if (hoveredTime === null || width === 0) return null;
    return ((hoveredTime - startTime) / (latestTime - startTime)) * width;
  });

  const timeLabels = [
    { label: '5m', pct: 0 },
    { label: '4m', pct: 0.2 },
    { label: '3m', pct: 0.4 },
    { label: '2m', pct: 0.6 },
    { label: '1m', pct: 0.8 },
    { label: 'now', pct: 1.0 }
  ];

  function getPath(data: Array<{timestamp: number, value: number}>) {
    if (data.length === 0 || width === 0) return '';
    const points = data.map(d => {
      const x = ((d.timestamp - startTime) / (latestTime - startTime)) * width;
      const y = height - (d.value / 100) * height;
      return `${x},${y}`;
    });
    return `M ${points.join(' L ')}`;
  }
</script>

<div class="chart-wrapper">
  <div class="header">
    <div class="label-container">
      <span class="label-text">CPU CORES ({coresArray.length})</span>
    </div>
    <div class="value-container">
      <span class="value-text">{overallCurrentValue.toFixed(1)}%</span>
    </div>
  </div>

  <div class="content">
    <div class="svg-container" bind:clientWidth={width}>
      <svg
        {width}
        height={height + 20}
        class="chart"
        onmousemove={(e) => mouseX = e.offsetX}
        onmouseleave={() => mouseX = null}
        role="img"
        aria-label="CPU Cores Chart"
      >
        {#if width > 0}
          {#each [0, 0.25, 0.5, 0.75, 1] as pct}
            {@const y = height - pct * height}
            {@const val = Math.round(pct * 100)}
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
                {val}%
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

          {#each coresArray as [core, data]}
            {@const color = colors[core % colors.length]}
            <path
              d={getPath(data)}
              fill="none"
              stroke={color}
              stroke-width="1.5"
              stroke-linejoin="round"
              stroke-linecap="round"
            />
          {/each}

          {#if hoverX !== null}
            <line
              x1={hoverX}
              y1="0"
              x2={hoverX}
              y2={height}
              stroke="var(--text-secondary)"
              stroke-width="1"
              class="crosshair"
            />
          {/if}
        {/if}
      </svg>
    </div>

    <div class="legend-panel" style="max-height: {height + 20}px;">
      {#each coresArray as [core]}
        {@const color = colors[core % colors.length]}
        {@const val = legendValues.get(core) || 0}
        <div class="legend-item">
          <div class="legend-label">
            <span class="dot" style="background-color: {color}"></span>
            <span>{core}</span>
          </div>
          <span class="legend-value">{val.toFixed(1)}%</span>
        </div>
      {/each}
    </div>
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
    color: var(--text-primary);
  }

  .content {
    display: flex;
    flex-direction: row;
    gap: var(--space-4);
  }

  .svg-container {
    flex: 1;
    position: relative;
    cursor: crosshair;
    min-width: 0;
  }

  .chart {
    display: block;
    overflow: visible;
  }

  .legend-panel {
    width: 90px;
    display: flex;
    flex-direction: column;
    gap: var(--space-compact, 6px);
    overflow-y: auto;
    padding-right: 4px;
  }

  .legend-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-family: var(--font-mono);
    font-size: var(--font-xs);
  }

  .legend-label {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--text-secondary);
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .legend-value {
    color: var(--text-primary);
    font-weight: 500;
    text-align: right;
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

  .crosshair {
    opacity: 0.7;
    stroke-dasharray: 4, 4;
  }
</style>

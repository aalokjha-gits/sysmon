<script lang="ts">
  import { fetchHistory } from '$lib/stores/metrics';

  type RangeKey = 'live' | '1h' | '6h' | '24h' | '7d' | '30d';

  interface Props {
    data: Array<{ timestamp: number; value: number }>;
    label: string;
    metric?: string;
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
    metric,
    unit = '%',
    color = 'var(--cpu-color)',
    maxPoints = 150,
    minValue = 0,
    maxValue = 100,
    height = 120
  }: Props = $props();

  const ranges: { key: RangeKey; label: string }[] = [
    { key: 'live', label: 'Live' },
    { key: '1h', label: '1h' },
    { key: '6h', label: '6h' },
    { key: '24h', label: '24h' },
    { key: '7d', label: '7d' },
    { key: '30d', label: '30d' }
  ];

  const timeLabelsByRange: Record<RangeKey, Array<{ label: string; pct: number }>> = {
    live: [
      { label: '5m', pct: 0 }, { label: '4m', pct: 0.2 }, { label: '3m', pct: 0.4 },
      { label: '2m', pct: 0.6 }, { label: '1m', pct: 0.8 }, { label: 'now', pct: 1.0 }
    ],
    '1h': [
      { label: '60m', pct: 0 }, { label: '48m', pct: 0.2 }, { label: '36m', pct: 0.4 },
      { label: '24m', pct: 0.6 }, { label: '12m', pct: 0.8 }, { label: 'now', pct: 1.0 }
    ],
    '6h': [
      { label: '6h', pct: 0 }, { label: '5h', pct: 1/6 }, { label: '4h', pct: 2/6 },
      { label: '3h', pct: 3/6 }, { label: '2h', pct: 4/6 }, { label: '1h', pct: 5/6 }, { label: 'now', pct: 1.0 }
    ],
    '24h': [
      { label: '24h', pct: 0 }, { label: '18h', pct: 0.25 }, { label: '12h', pct: 0.5 },
      { label: '6h', pct: 0.75 }, { label: 'now', pct: 1.0 }
    ],
    '7d': [
      { label: '7d', pct: 0 }, { label: '6d', pct: 1/7 }, { label: '5d', pct: 2/7 },
      { label: '4d', pct: 3/7 }, { label: '3d', pct: 4/7 }, { label: '2d', pct: 5/7 },
      { label: '1d', pct: 6/7 }, { label: 'now', pct: 1.0 }
    ],
    '30d': [
      { label: '30d', pct: 0 }, { label: '25d', pct: 1/6 }, { label: '20d', pct: 2/6 },
      { label: '15d', pct: 3/6 }, { label: '10d', pct: 4/6 }, { label: '5d', pct: 5/6 }, { label: 'now', pct: 1.0 }
    ]
  };

  let selectedRange = $state<RangeKey>('live');
  let historyData = $state<Array<{ timestamp: number; value: number }>>([]);
  let loading = $state(false);

  let displayData = $derived(selectedRange === 'live' ? data : historyData);

  let timeLabels = $derived(timeLabelsByRange[selectedRange]);

  let latestTime = $derived.by(() => {
    if (selectedRange === 'live') {
      return displayData.length > 0 ? displayData[displayData.length - 1].timestamp : Date.now();
    }
    return displayData.length > 0 ? displayData[displayData.length - 1].timestamp : Date.now();
  });

  let startTime = $derived.by(() => {
    if (selectedRange === 'live') {
      return latestTime - (maxPoints * 2000);
    }
    return displayData.length > 0 ? displayData[0].timestamp : latestTime;
  });

  let currentValue = $derived(displayData.length > 0 ? displayData[displayData.length - 1].value : 0);

  let width = $state(0);
  let mouseX = $state<number | null>(null);

  let dynamicMaxValue = $derived.by(() => {
    if (selectedRange === 'live') return maxValue;
    if (displayData.length === 0) return maxValue;
    const maxDataVal = Math.max(...displayData.map(d => d.value));
    if (maxDataVal <= maxValue) return maxValue;
    return Math.ceil(maxDataVal / 10) * 10;
  });

  let pathData = $derived.by(() => {
    if (displayData.length === 0 || width === 0) return '';
    const range = latestTime - startTime;
    if (range === 0) return '';
    const points = displayData.map(d => {
      const x = ((d.timestamp - startTime) / range) * width;
      const y = height - ((d.value - minValue) / (dynamicMaxValue - minValue)) * height;
      return `${x},${y}`;
    });
    return `M ${points.join(' L ')}`;
  });

  let fillPathData = $derived.by(() => {
    if (!pathData || displayData.length === 0 || width === 0) return '';
    const range = latestTime - startTime;
    if (range === 0) return '';
    const firstX = ((displayData[0].timestamp - startTime) / range) * width;
    const lastX = ((displayData[displayData.length - 1].timestamp - startTime) / range) * width;
    return `${pathData} L ${lastX},${height} L ${firstX},${height} Z`;
  });

  let hoveredPoint = $derived.by(() => {
    if (mouseX === null || width === 0 || displayData.length === 0) return null;
    const range = latestTime - startTime;
    if (range === 0) return null;
    const timeAtMouse = startTime + (mouseX / width) * range;

    let closest = displayData[0];
    let minDiff = Math.abs(displayData[0].timestamp - timeAtMouse);

    for (let i = 1; i < displayData.length; i++) {
      const diff = Math.abs(displayData[i].timestamp - timeAtMouse);
      if (diff < minDiff) {
        minDiff = diff;
        closest = displayData[i];
      }
    }
    return closest;
  });

  async function selectRange(range: RangeKey) {
    selectedRange = range;
    if (range === 'live' || !metric) {
      historyData = [];
      return;
    }
    loading = true;
    try {
      const resp = await fetchHistory(metric, range);
      historyData = resp.points.map(p => ({
        timestamp: p.timestamp * 1000,
        value: p.value
      }));
    } catch {
      historyData = [];
    } finally {
      loading = false;
    }
  }

  let gradientId = $derived(`gradient-${label.toLowerCase().replace(/[^a-z0-9]/g, '-')}-${Math.random().toString(36).slice(2, 7)}`);
</script>

<div class="chart-wrapper">
  <div class="header">
    <div class="label-container">
      <span class="label-text">{label}</span>
      {#if loading}
        <span class="loading-dot"></span>
      {/if}
    </div>
    <div class="header-right">
      {#if metric}
        <div class="range-pills">
          {#each ranges as r}
            <button
              class="range-pill"
              class:active={selectedRange === r.key}
              style:--pill-color={color}
              onclick={() => selectRange(r.key)}
            >
              {r.label}
            </button>
          {/each}
        </div>
      {/if}
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
          {@const val = minValue + pct * (dynamicMaxValue - minValue)}
          <line
            x1="0" y1={y} x2={width} y2={y}
            stroke="var(--border-color)" stroke-dasharray="4, 4" stroke-width="1"
            class="grid-line"
          />
          {#if pct > 0}
            <text x="0" y={y > 10 ? y - 4 : y + 12} class="axis-text">
              {val.toFixed(unit === '°C' ? 0 : 0)}{unit}
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
          <path d={fillPathData} fill="url(#{gradientId})" stroke="none" />
          <path
            d={pathData} fill="none" stroke={color}
            stroke-width="2" stroke-linejoin="round" stroke-linecap="round"
          />
        {/if}

        {#if hoveredPoint}
          {@const hrange = latestTime - startTime}
          {@const hx = hrange > 0 ? ((hoveredPoint.timestamp - startTime) / hrange) * width : 0}
          {@const hy = height - ((hoveredPoint.value - minValue) / (dynamicMaxValue - minValue)) * height}

          <line
            x1={hx} y1="0" x2={hx} y2={height}
            stroke="var(--text-secondary)" stroke-width="1" class="crosshair"
          />

          <circle
            cx={hx} cy={hy} r="4" fill={color}
            stroke="var(--bg-primary)" stroke-width="2"
          />

          {@const tooltipWidth = 80}
          {@const tooltipHeight = 40}
          {@const tx = hx > width - tooltipWidth - 10 ? hx - tooltipWidth - 10 : hx + 10}
          {@const ty = hy < tooltipHeight + 10 ? hy + 10 : hy - tooltipHeight - 10}

          <g transform="translate({tx}, {ty})" class="tooltip">
            <rect
              x="0" y="0" width={tooltipWidth} height={tooltipHeight} rx="4"
              fill="var(--bg-surface-hover)" stroke="var(--border-color)" stroke-width="1"
            />
            <text x="6" y="16" class="tooltip-time">
              {selectedRange === 'live'
                ? new Date(hoveredPoint.timestamp).toLocaleTimeString([], {hour12: false, hour: '2-digit', minute:'2-digit', second:'2-digit'})
                : selectedRange === '7d' || selectedRange === '30d'
                  ? new Date(hoveredPoint.timestamp).toLocaleDateString([], {month: 'short', day: 'numeric'}) + ' ' + new Date(hoveredPoint.timestamp).toLocaleTimeString([], {hour12: false, hour: '2-digit', minute:'2-digit'})
                  : new Date(hoveredPoint.timestamp).toLocaleTimeString([], {hour12: false, hour: '2-digit', minute:'2-digit'})}
            </text>
            <text x="6" y="32" class="tooltip-value" fill={color}>
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
    gap: 8px;
  }

  .label-container {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .label-text {
    font-family: var(--font-mono);
    font-size: var(--font-sm);
    color: var(--text-secondary);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .loading-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
    animation: pulse 1s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 0.3; }
    50% { opacity: 1; }
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }

  .range-pills {
    display: flex;
    gap: 2px;
    background: var(--bg-elevated);
    border-radius: 4px;
    padding: 2px;
    flex-shrink: 0;
  }

  .range-pill {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 500;
    padding: 2px 6px;
    border: none;
    border-radius: 3px;
    cursor: pointer;
    color: var(--text-muted);
    background: transparent;
    transition: all var(--transition-fast);
    line-height: 1.4;
  }

  .range-pill:hover {
    color: var(--text-primary);
    background: var(--bg-surface);
  }

  .range-pill.active {
    color: var(--text-primary);
    background: var(--bg-surface);
    box-shadow: 0 1px 2px rgba(0,0,0,0.2);
  }

  .value-text {
    font-family: var(--font-mono);
    font-size: var(--font-lg);
    font-weight: 700;
    flex-shrink: 0;
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

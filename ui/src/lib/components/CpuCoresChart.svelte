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
  let sortByValue = $state(false);
  let selectedCores = $state<Set<number>>(new Set());

  let coresArray = $derived(Array.from(coreData.entries()).sort((a, b) => a[0] - b[0]));

  let hasSelection = $derived(selectedCores.size > 0);

  function toggleCore(core: number) {
    selectedCores = new Set(selectedCores);
    if (selectedCores.has(core)) {
      selectedCores.delete(core);
    } else {
      selectedCores.add(core);
    }
    selectedCores = selectedCores;
  }

  function clearSelection() {
    selectedCores = new Set();
  }

  function isCoreVisible(core: number): boolean {
    return !hasSelection || selectedCores.has(core);
  }

  function getCoreOpacity(core: number): number {
    if (!hasSelection) return 1;
    return selectedCores.has(core) ? 1 : 0.08;
  }

  function getCoreStrokeWidth(core: number): number {
    if (!hasSelection) return 1.5;
    return selectedCores.has(core) ? 2.5 : 1;
  }

  let sortedLegendEntries = $derived.by(() => {
    const entries = coresArray.map(([core]) => ({
      core,
      value: legendValues.get(core) || 0,
      color: colors[core % colors.length]
    }));
    if (sortByValue) {
      return entries.sort((a, b) => b.value - a.value);
    }
    return entries;
  });

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
    const range = latestTime - startTime;
    if (range === 0) return '';
    const points = data.map(d => {
      const x = ((d.timestamp - startTime) / range) * width;
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
      {#if hasSelection}
        <button class="clear-btn" onclick={clearSelection} title="Show all cores">
          ✕ clear
        </button>
      {/if}
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
              x1="0" y1={y} x2={width} y2={y}
              stroke="var(--border-color)" stroke-dasharray="4, 4" stroke-width="1"
              class="grid-line"
            />
            {#if pct > 0}
              <text x="0" y={y > 10 ? y - 4 : y + 12} class="axis-text">
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
              stroke-width={getCoreStrokeWidth(core)}
              stroke-linejoin="round"
              stroke-linecap="round"
              opacity={getCoreOpacity(core)}
              style="transition: opacity 0.2s ease, stroke-width 0.2s ease;"
            />
          {/each}

          {#if hoverX !== null}
            <line
              x1={hoverX} y1="0" x2={hoverX} y2={height}
              stroke="var(--text-secondary)" stroke-width="1" class="crosshair"
            />
          {/if}
        {/if}
      </svg>
    </div>

    <div class="legend-panel" style="max-height: {height + 20}px;">
      <button class="sort-toggle" onclick={() => sortByValue = !sortByValue} title={sortByValue ? 'Sorted by usage' : 'Sorted by core #'}>
        {sortByValue ? '▼ usage' : '# core'}
      </button>
      {#each sortedLegendEntries as entry}
        <button
          class="legend-item"
          class:dimmed={hasSelection && !selectedCores.has(entry.core)}
          class:selected={selectedCores.has(entry.core)}
          onclick={() => toggleCore(entry.core)}
          title="Click to {selectedCores.has(entry.core) ? 'deselect' : 'isolate'} Core {entry.core}"
        >
          <span class="dot" style="background-color: {entry.color}"></span>
          <span class="legend-label">Core {entry.core}</span>
          <span class="legend-sep">—</span>
          <span class="legend-value">{entry.value.toFixed(1)}%</span>
        </button>
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

  .label-container {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .label-text {
    font-family: var(--font-mono);
    font-size: var(--font-sm);
    color: var(--text-secondary);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .clear-btn {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-muted);
    background: var(--bg-elevated);
    border: 1px solid var(--border-color);
    border-radius: 3px;
    padding: 1px 6px;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .clear-btn:hover {
    color: var(--text-primary);
    border-color: var(--accent);
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
    width: 190px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow-y: auto;
    padding-right: 4px;
    flex-shrink: 0;
  }

  .sort-toggle {
    background: var(--bg-elevated);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: var(--font-xs);
    padding: 3px 8px;
    cursor: pointer;
    align-self: flex-end;
    margin-bottom: 4px;
    transition: color var(--transition-fast), border-color var(--transition-fast), background var(--transition-fast);
    font-weight: 500;
  }

  .sort-toggle:hover {
    color: var(--text-primary);
    border-color: var(--accent);
    background: var(--bg-surface);
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 5px;
    font-family: var(--font-mono);
    font-size: var(--font-xs);
    white-space: nowrap;
    border: none;
    background: transparent;
    padding: 2px 4px;
    border-radius: 3px;
    cursor: pointer;
    transition: all 0.15s ease;
    width: 100%;
    text-align: left;
  }

  .legend-item:hover {
    background: var(--bg-elevated);
  }

  .legend-item.selected {
    background: var(--bg-elevated);
  }

  .legend-item.dimmed {
    opacity: 0.3;
  }

  .legend-item.dimmed:hover {
    opacity: 0.7;
  }

  .legend-label {
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .legend-sep {
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .legend-value {
    color: var(--text-primary);
    font-weight: 500;
    text-align: right;
    margin-left: auto;
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

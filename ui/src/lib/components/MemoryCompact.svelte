<script lang="ts">
  import { metrics, memoryPercent } from '$lib/stores/metrics';
  import { formatBytes } from '$lib/utils/format';

  function getBarColor(percent: number): string {
    if (percent >= 80) return 'var(--danger)';
    if (percent >= 60) return 'var(--warning)';
    if (percent >= 40) return 'var(--accent)';
    return 'var(--success)';
  }
</script>

<div class="memory-compact compact-panel">
  <div class="panel-header">
    <span class="compact-panel-title">Memory</span>
    <span class="mem-percent" style:color={getBarColor($memoryPercent)}>
      {$memoryPercent.toFixed(1)}%
    </span>
  </div>

  <div class="progress-bar">
    <div
      class="progress-fill"
      style:width="{$memoryPercent}%"
      style:background-color={getBarColor($memoryPercent)}
    ></div>
  </div>

  <div class="mem-details">
    {#if $metrics}
      <span class="mono">
        {formatBytes($metrics.memory.used_bytes)} / {formatBytes($metrics.memory.total_bytes)}
      </span>
      {#if $metrics.memory.swap_total_bytes > 0}
        <span class="swap-info">
          Swap: <span class="mono">{formatBytes($metrics.memory.swap_used_bytes)}</span>
        </span>
      {/if}
    {:else}
      <span class="mono">-- / --</span>
    {/if}
  </div>
</div>

<style>
  .compact-panel {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    padding: 8px 10px;
  }

  .compact-panel-title {
    font-size: var(--font-xs);
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 6px;
  }

  .mem-percent {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: var(--font-base);
    font-weight: 600;
  }

  .progress-bar {
    height: 6px;
    background: var(--bg-elevated);
    border-radius: 3px;
    overflow: hidden;
    margin-bottom: 6px;
  }

  .progress-fill {
    height: 100%;
    border-radius: 3px;
    transition: width 300ms ease;
  }

  .mem-details {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: var(--font-xs);
    color: var(--text-secondary);
  }

  .mono {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }

  .swap-info {
    color: var(--text-muted);
  }

  .swap-info .mono {
    color: var(--text-secondary);
  }
</style>

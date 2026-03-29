<script lang="ts">
  import { metrics, cpuPercent, memoryPercent, loadAvg, coreCount } from '$lib/stores/metrics';
  import { formatPercent, formatBytes, getPercentColor } from '$lib/utils/format';

  function getLoadColor(load: number, cores: number): string {
    const ratio = load / Math.max(cores, 1);
    if (ratio >= 2) return 'var(--danger)';
    if (ratio >= 1) return 'var(--warning)';
    return 'var(--success)';
  }

  function getLoadStatus(load: number, cores: number): string {
    const ratio = load / Math.max(cores, 1);
    if (ratio >= 2) return 'critical';
    if (ratio >= 1) return 'warning';
    return 'normal';
  }
</script>

<div class="system-overview">
  <!-- CPU Card -->
  <div class="metric-card">
    <div class="card-header">
      <div class="icon-wrapper cpu">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="4" y="4" width="16" height="16" rx="2" />
          <rect x="9" y="9" width="6" height="6" />
          <path d="M9 1v3M15 1v3M9 20v3M15 20v3M20 9h3M20 15h3M1 9h3M1 15h3" />
        </svg>
      </div>
      <span class="card-label">CPU Usage</span>
    </div>
    <div class="card-body">
      <span class="metric-value" style:color={getPercentColor($cpuPercent)}>
        {formatPercent($cpuPercent)}
      </span>
      <div class="metric-bar">
        <div
          class="metric-fill cpu"
          style:width="{Math.min($cpuPercent, 100)}%"
          style:background-color={getPercentColor($cpuPercent)}
        ></div>
      </div>
    </div>
    <div class="card-footer">
      <span>{$metrics?.cpu.cores.length ?? 0} cores</span>
    </div>
  </div>

  <!-- Memory Card -->
  <div class="metric-card">
    <div class="card-header">
      <div class="icon-wrapper memory">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="4" y="4" width="16" height="16" rx="2" />
          <path d="M9 9h6v6H9z" />
          <path d="M9 1v3M15 1v3M9 20v3M15 20v3M20 9h3M20 15h3M1 9h3M1 15h3" />
        </svg>
      </div>
      <span class="card-label">Memory</span>
    </div>
    <div class="card-body">
      <span class="metric-value" style:color={getPercentColor($memoryPercent)}>
        {formatPercent($memoryPercent)}
      </span>
      <div class="metric-bar">
        <div
          class="metric-fill memory"
          style:width="{Math.min($memoryPercent, 100)}%"
          style:background-color={getPercentColor($memoryPercent)}
        ></div>
      </div>
    </div>
    <div class="card-footer">
      {#if $metrics}
        <span>{formatBytes($metrics.memory.used_bytes)} / {formatBytes($metrics.memory.total_bytes)}</span>
      {:else}
        <span>-- / --</span>
      {/if}
    </div>
  </div>

  <!-- Load Average Card -->
  <div class="metric-card">
    <div class="card-header">
      <div class="icon-wrapper load">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 20V10M12 20V4M6 20v-6" />
        </svg>
      </div>
      <span class="card-label">Load Average</span>
    </div>
    <div class="card-body load-body">
      {#each $loadAvg as load, index}
        <div class="load-item">
          <span class="load-label">{[1, 5, 15][index]}m</span>
          <span
            class="load-value"
            style:color={getLoadColor(load, $coreCount)}
            class:critical={getLoadStatus(load, $coreCount) === 'critical'}
          >
            {load.toFixed(2)}
          </span>
        </div>
      {/each}
    </div>
    <div class="card-footer">
      <span>{$coreCount} core{$coreCount !== 1 ? 's' : ''}</span>
    </div>
  </div>
</div>

<style>
  .system-overview {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-4);
    padding: var(--space-4) var(--space-6);
  }

  .metric-card {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
    transition: border-color var(--transition-fast), transform var(--transition-fast);
  }

  .metric-card:hover {
    border-color: var(--border-color);
    transform: translateY(-2px);
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-bottom: var(--space-3);
  }

  .icon-wrapper {
    width: 36px;
    height: 36px;
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .icon-wrapper svg {
    width: 20px;
    height: 20px;
  }

  .icon-wrapper.cpu {
    background: rgba(59, 130, 246, 0.15);
    color: var(--cpu-color);
  }

  .icon-wrapper.memory {
    background: rgba(139, 92, 246, 0.15);
    color: var(--memory-color);
  }

  .icon-wrapper.load {
    background: rgba(34, 197, 94, 0.15);
    color: var(--success);
  }

  .card-label {
    font-size: 0.875rem;
    color: var(--text-secondary);
    font-weight: 500;
  }

  .card-body {
    margin-bottom: var(--space-3);
  }

  .metric-value {
    font-size: 2.5rem;
    font-weight: 700;
    line-height: 1;
    display: block;
    margin-bottom: var(--space-3);
    transition: color var(--transition-normal);
  }

  .metric-bar {
    height: 6px;
    background: var(--bg-elevated);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .metric-fill {
    height: 100%;
    border-radius: var(--radius-sm);
    transition: width var(--transition-normal), background-color var(--transition-normal);
  }

  .load-body {
    display: flex;
    gap: var(--space-4);
    align-items: center;
    min-height: 76px;
  }

  .load-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-1);
  }

  .load-label {
    font-size: 0.75rem;
    color: var(--text-muted);
    font-weight: 500;
  }

  .load-value {
    font-size: 1.75rem;
    font-weight: 700;
    transition: color var(--transition-normal);
  }

  .load-value.critical {
    animation: pulse 2s infinite;
  }

  .card-footer {
    font-size: 0.8125rem;
    color: var(--text-muted);
    padding-top: var(--space-3);
    border-top: 1px solid var(--border-color);
  }

  @media (max-width: 900px) {
    .system-overview {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  @media (max-width: 640px) {
    .system-overview {
      grid-template-columns: 1fr;
      padding: var(--space-3) var(--space-4);
    }

    .metric-value {
      font-size: 2rem;
    }

    .load-value {
      font-size: 1.5rem;
    }
  }
</style>

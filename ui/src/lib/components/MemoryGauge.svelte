<script lang="ts">
  import { metrics, memoryPercent } from '$lib/stores/metrics';
  import { formatBytes, formatPercent } from '$lib/utils/format';

  function getGaugeColor(percent: number): string {
    if (percent >= 80) return '#ef4444';
    if (percent >= 60) return '#f59e0b';
    if (percent >= 40) return '#3b82f6';
    return '#22c55e';
  }

  function getStrokeDashoffset(percent: number): number {
    const circumference = 2 * Math.PI * 45; // radius = 45
    return circumference - (Math.min(percent, 100) / 100) * circumference;
  }

  const circumference = 2 * Math.PI * 45;
  const strokeDashoffset = $derived(getStrokeDashoffset($memoryPercent));
  const gaugeColor = $derived(getGaugeColor($memoryPercent));
</script>

<div class="memory-gauge">
  <div class="gauge-header">
    <h3 class="gauge-title">
      <svg class="title-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="4" y="4" width="16" height="16" rx="2" />
        <path d="M9 9h6v6H9z" />
        <path d="M9 1v3M15 1v3M9 20v3M15 20v3M20 9h3M20 15h3M1 9h3M1 15h3" />
      </svg>
      Memory Usage
    </h3>
  </div>

  <div class="gauge-container">
    <svg class="gauge-svg" viewBox="0 0 120 120">
      <!-- Background circle -->
      <circle
        class="gauge-bg"
        cx="60"
        cy="60"
        r="45"
        fill="none"
        stroke="var(--bg-elevated)"
        stroke-width="10"
      />
      <!-- Progress circle -->
      <circle
        class="gauge-progress"
        cx="60"
        cy="60"
        r="45"
        fill="none"
        stroke={gaugeColor}
        stroke-width="10"
        stroke-linecap="round"
        stroke-dasharray={circumference}
        style:stroke-dashoffset={strokeDashoffset}
        style:filter={`drop-shadow(0 0 8px ${gaugeColor}40)`}
        transform="rotate(-90 60 60)"
      />
    </svg>
    <div class="gauge-center">
      <span class="gauge-percent" style:color={gaugeColor}>
        {formatPercent($memoryPercent)}
      </span>
      <span class="gauge-label">used</span>
    </div>
  </div>

  <div class="gauge-stats">
    <div class="stat-item">
      <span class="stat-label">Used</span>
      <span class="stat-value">{$metrics ? formatBytes($metrics.memory.used_bytes) : '--'}</span>
    </div>
    <div class="stat-item">
      <span class="stat-label">Total</span>
      <span class="stat-value">{$metrics ? formatBytes($metrics.memory.total_bytes) : '--'}</span>
    </div>
    {#if $metrics && $metrics.memory.swap_total_bytes > 0}
      <div class="stat-item">
        <span class="stat-label">Swap</span>
        <span class="stat-value" class:swap-used={$metrics.memory.swap_used_bytes > 0}>
          {formatBytes($metrics.memory.swap_used_bytes)} / {formatBytes($metrics.memory.swap_total_bytes)}
        </span>
      </div>
    {/if}
  </div>
</div>

<style>
  .memory-gauge {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
  }

  .gauge-header {
    margin-bottom: var(--space-4);
  }

  .gauge-title {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .title-icon {
    width: 18px;
    height: 18px;
    color: var(--memory-color);
  }

  .gauge-container {
    position: relative;
    width: 160px;
    height: 160px;
    margin: 0 auto var(--space-4);
  }

  .gauge-svg {
    width: 100%;
    height: 100%;
    transform: rotate(0deg);
  }

  .gauge-bg {
    transition: stroke var(--transition-fast);
  }

  .gauge-progress {
    transition: stroke-dashoffset var(--transition-normal), stroke var(--transition-normal);
  }

  .gauge-center {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    text-align: center;
  }

  .gauge-percent {
    font-size: 2rem;
    font-weight: 700;
    display: block;
    line-height: 1;
    transition: color var(--transition-normal);
  }

  .gauge-label {
    font-size: 0.75rem;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .gauge-stats {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding-top: var(--space-3);
    border-top: 1px solid var(--border-color);
  }

  .stat-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.875rem;
  }

  .stat-label {
    color: var(--text-secondary);
  }

  .stat-value {
    color: var(--text-primary);
    font-weight: 500;
  }

  .stat-value.swap-used {
    color: var(--warning);
  }

  @media (max-width: 640px) {
    .memory-gauge {
      padding: var(--space-3);
    }

    .gauge-container {
      width: 140px;
      height: 140px;
    }

    .gauge-percent {
      font-size: 1.75rem;
    }
  }
</style>

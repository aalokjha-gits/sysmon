<script lang="ts">
  import { metrics, coreCount } from '$lib/stores/metrics';
  import { formatPercent, getPercentColor } from '$lib/utils/format';

  function getBarColor(usage: number): string {
    if (usage >= 80) return 'var(--danger)';
    if (usage >= 60) return 'var(--warning)';
    if (usage >= 40) return 'var(--accent)';
    return 'var(--success)';
  }

  function getBarGradient(usage: number): string {
    if (usage >= 80) {
      return 'linear-gradient(90deg, #ef4444 0%, #f87171 100%)';
    }
    if (usage >= 60) {
      return 'linear-gradient(90deg, #f59e0b 0%, #fbbf24 100%)';
    }
    if (usage >= 40) {
      return 'linear-gradient(90deg, #3b82f6 0%, #60a5fa 100%)';
    }
    return 'linear-gradient(90deg, #22c55e 0%, #4ade80 100%)';
  }
</script>

<div class="cpu-chart">
  <div class="chart-header">
    <h3 class="chart-title">
      <svg class="title-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="4" y="4" width="16" height="16" rx="2" />
        <rect x="9" y="9" width="6" height="6" />
        <path d="M9 1v3M15 1v3M9 20v3M15 20v3M20 9h3M20 15h3M1 9h3M1 15h3" />
      </svg>
      CPU Cores
    </h3>
    <span class="core-count">{$coreCount} cores</span>
  </div>

  <div class="cores-grid">
    {#if $metrics}
      {#each $metrics.cpu.cores as core (core.id)}
        <div class="core-item">
          <span class="core-label">{core.id}</span>
          <div class="core-bar-container">
            <div
              class="core-bar"
              style:width="{Math.min(core.usage_percent, 100)}%"
              style:background={getBarGradient(core.usage_percent)}
            ></div>
          </div>
          <span class="core-value" style:color={getBarColor(core.usage_percent)}>
            {formatPercent(core.usage_percent)}
          </span>
          {#if core.frequency_mhz > 0}
            <span class="core-freq">{core.frequency_mhz}MHz</span>
          {/if}
        </div>
      {/each}
    {:else}
      {#each Array(4) as _, i}
        <div class="core-item skeleton">
          <span class="core-label">{i}</span>
          <div class="core-bar-container">
            <div class="core-bar" style:width="0%"></div>
          </div>
          <span class="core-value">--%</span>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .cpu-chart {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
  }

  .chart-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-4);
  }

  .chart-title {
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
    color: var(--cpu-color);
  }

  .core-count {
    font-size: 0.8125rem;
    color: var(--text-muted);
    font-weight: 500;
  }

  .cores-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: var(--space-3);
  }

  .core-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: var(--bg-elevated);
    border-radius: var(--radius-md);
    transition: background var(--transition-fast);
  }

  .core-item:hover {
    background: var(--bg-surface-hover);
  }

  .core-item.skeleton {
    opacity: 0.5;
  }

  .core-label {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-muted);
    min-width: 24px;
    text-align: center;
    background: var(--bg-surface);
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
  }

  .core-bar-container {
    flex: 1;
    height: 8px;
    background: var(--bg-surface);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .core-bar {
    height: 100%;
    border-radius: var(--radius-sm);
    transition: width var(--transition-normal), background var(--transition-normal);
    min-width: 2px;
  }

  .core-value {
    font-size: 0.8125rem;
    font-weight: 600;
    min-width: 44px;
    text-align: right;
    transition: color var(--transition-normal);
  }

  .core-freq {
    font-size: 0.6875rem;
    color: var(--text-muted);
    min-width: 50px;
    text-align: right;
  }

  @media (max-width: 1200px) {
    .cores-grid {
      grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    }
  }

  @media (max-width: 640px) {
    .cpu-chart {
      padding: var(--space-3);
    }

    .cores-grid {
      grid-template-columns: 1fr;
    }

    .core-freq {
      display: none;
    }
  }
</style>

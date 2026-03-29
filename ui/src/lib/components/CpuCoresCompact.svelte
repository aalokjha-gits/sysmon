<script lang="ts">
  import { metrics, coreCount } from '$lib/stores/metrics';
  import { formatPercent } from '$lib/utils/format';

  function getBarColor(usage: number): string {
    if (usage >= 80) return 'var(--danger)';
    if (usage >= 60) return 'var(--warning)';
    if (usage >= 40) return 'var(--accent)';
    return 'var(--success)';
  }
</script>

<div class="cpu-cores-compact">
  <div class="cores-header">
    <span class="section-title">CPU Cores ({$coreCount})</span>
    {#if $metrics}
      <span class="overall-usage" style:color={getBarColor($metrics.cpu.overall_percent)}>
        {formatPercent($metrics.cpu.overall_percent)}
      </span>
    {/if}
  </div>

  <div class="cores-grid">
    {#if $metrics}
      {#each $metrics.cpu.cores as core (core.id)}
        <div class="core-cell" title="Core {core.id}: {core.frequency_mhz > 0 ? core.frequency_mhz + ' MHz' : ''}">
          <span class="core-id">{core.id}</span>
          <div class="core-bar-bg">
            <div
              class="core-bar-fill"
              style:width="{Math.min(core.usage_percent, 100)}%"
              style:background-color={getBarColor(core.usage_percent)}
            ></div>
          </div>
          <span class="core-pct" style:color={getBarColor(core.usage_percent)}>
            {Math.round(core.usage_percent)}%
          </span>
        </div>
      {/each}
    {:else}
      {#each Array(8) as _, i}
        <div class="core-cell skeleton">
          <span class="core-id">{i}</span>
          <div class="core-bar-bg">
            <div class="core-bar-fill" style:width="0%"></div>
          </div>
          <span class="core-pct">--</span>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .cpu-cores-compact {
    padding: 8px 10px;
    border-bottom: 1px solid var(--border-color);
  }

  .cores-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 6px;
  }

  .section-title {
    font-size: var(--font-xs);
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .overall-usage {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: var(--font-sm);
    font-weight: 600;
  }

  .cores-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(110px, 1fr));
    gap: 3px;
  }

  .core-cell {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 3px 5px;
    background: var(--bg-elevated);
    border-radius: 3px;
    font-size: var(--font-xs);
  }

  .core-cell.skeleton {
    opacity: 0.5;
  }

  .core-id {
    color: var(--text-muted);
    font-weight: 500;
    min-width: 14px;
  }

  .core-bar-bg {
    flex: 1;
    height: 3px;
    background: var(--bg-surface);
    border-radius: 2px;
    overflow: hidden;
  }

  .core-bar-fill {
    height: 100%;
    border-radius: 2px;
    transition: width 300ms ease, background-color 300ms ease;
    min-width: 1px;
  }

  .core-pct {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    min-width: 22px;
    text-align: right;
    font-weight: 500;
  }
</style>

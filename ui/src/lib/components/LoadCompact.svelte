<script lang="ts">
  import { loadAvg, coreCount } from '$lib/stores/metrics';

  function getLoadColor(load: number, cores: number): string {
    const ratio = load / Math.max(cores, 1);
    if (ratio >= 2) return 'var(--danger)';
    if (ratio >= 1) return 'var(--warning)';
    return 'var(--success)';
  }
</script>

<div class="load-compact compact-panel">
  <div class="panel-header">
    <span class="compact-panel-title">Load Average</span>
  </div>

  <div class="load-values">
    {#each $loadAvg as load, index}
      <div class="load-item">
        <span class="load-period">{[1, 5, 15][index]}M</span>
        <span class="load-number mono" style:color={getLoadColor(load, $coreCount)}>
          {load.toFixed(2)}
        </span>
      </div>
    {/each}
  </div>

  <div class="load-max">
    {$coreCount} cores = <span class="mono">{$coreCount}.00</span> max normal
  </div>
</div>

<style>
  .compact-panel {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    padding: 8px 10px;
    overflow: hidden;
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

  .load-values {
    display: flex;
    gap: 16px;
    margin-bottom: 6px;
  }

  .load-item {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .load-period {
    font-size: var(--font-xs);
    color: var(--text-muted);
  }

  .load-number {
    font-size: var(--font-base);
    font-weight: 600;
  }

  .load-max {
    font-size: var(--font-xs);
    color: var(--text-muted);
  }

  .mono {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
</style>

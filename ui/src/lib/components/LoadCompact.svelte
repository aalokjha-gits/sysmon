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
  <div class="load-line">
    <span class="compact-panel-title">Load</span>
    <div class="load-values">
      {#each $loadAvg as load, index}
        <span class="load-entry">
          <span class="load-period">{[1, 5, 15][index]}m</span>
          <span class="load-number mono" style:color={getLoadColor(load, $coreCount)}>
            {load.toFixed(2)}
          </span>
        </span>
      {/each}
    </div>
    <span class="load-max">{$coreCount}c max {$coreCount}.00</span>
  </div>
</div>

<style>
  .compact-panel {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    padding: 6px 10px;
    overflow: hidden;
  }

  .compact-panel-title {
    font-size: var(--font-xs);
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    flex-shrink: 0;
  }

  .load-line {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .load-values {
    display: flex;
    gap: 8px;
  }

  .load-entry {
    display: flex;
    align-items: center;
    gap: 3px;
  }

  .load-period {
    font-size: 10px;
    color: var(--text-muted);
  }

  .load-number {
    font-size: var(--font-sm);
    font-weight: 600;
  }

  .load-max {
    font-size: 10px;
    color: var(--text-muted);
    margin-left: auto;
    white-space: nowrap;
  }

  .mono {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
</style>

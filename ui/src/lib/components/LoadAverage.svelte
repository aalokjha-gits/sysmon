<script lang="ts">
  import { loadAvg, coreCount } from '$lib/stores/metrics';

  function getLoadColor(load: number, cores: number): string {
    const ratio = load / Math.max(cores, 1);
    if (ratio >= 2) return 'var(--danger)';
    if (ratio >= 1) return 'var(--warning)';
    return 'var(--success)';
  }

  function getLoadBg(load: number, cores: number): string {
    const ratio = load / Math.max(cores, 1);
    if (ratio >= 2) return 'rgba(239, 68, 68, 0.15)';
    if (ratio >= 1) return 'rgba(245, 158, 11, 0.15)';
    return 'rgba(34, 197, 94, 0.15)';
  }
</script>

<div class="load-average">
  <div class="load-header">
    <h3 class="load-title">
      <svg class="title-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M18 20V10M12 20V4M6 20v-6" />
      </svg>
      Load Average
    </h3>
  </div>

  <div class="load-badges">
    {#each $loadAvg as load, index}
      <div
        class="load-badge"
        style:background-color={getLoadBg(load, $coreCount)}
        style:border-color={getLoadColor(load, $coreCount)}
      >
        <span class="load-period">{[1, 5, 15][index]}m</span>
        <span class="load-number" style:color={getLoadColor(load, $coreCount)}>
          {load.toFixed(2)}
        </span>
      </div>
    {/each}
  </div>

  <div class="load-info">
    <span class="info-text">
      {$coreCount} core{$coreCount !== 1 ? 's' : ''} =
      <span style:color={$loadAvg[0] > $coreCount ? 'var(--warning)' : 'var(--success)'}>
        {$coreCount}.00
      </span>
      max normal load
    </span>
  </div>
</div>

<style>
  .load-average {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
  }

  .load-header {
    margin-bottom: var(--space-4);
  }

  .load-title {
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
    color: var(--success);
  }

  .load-badges {
    display: flex;
    gap: var(--space-3);
    justify-content: center;
    margin-bottom: var(--space-4);
  }

  .load-badge {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-lg);
    border: 1px solid;
    min-width: 80px;
    transition: all var(--transition-fast);
  }

  .load-period {
    font-size: 0.75rem;
    color: var(--text-muted);
    font-weight: 500;
    text-transform: uppercase;
    margin-bottom: var(--space-1);
  }

  .load-number {
    font-size: 1.75rem;
    font-weight: 700;
    transition: color var(--transition-normal);
  }

  .load-info {
    padding-top: var(--space-3);
    border-top: 1px solid var(--border-color);
    text-align: center;
  }

  .info-text {
    font-size: 0.8125rem;
    color: var(--text-secondary);
  }

  @media (max-width: 640px) {
    .load-average {
      padding: var(--space-3);
    }

    .load-badges {
      gap: var(--space-2);
    }

    .load-badge {
      min-width: 60px;
      padding: var(--space-2) var(--space-3);
    }

    .load-number {
      font-size: 1.5rem;
    }
  }
</style>

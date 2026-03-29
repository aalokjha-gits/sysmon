<script lang="ts">
  import { diskMetrics } from '$lib/stores/metrics';
  import { formatBytes } from '$lib/utils/format';

  function getUsageColor(percent: number): string {
    if (percent >= 90) return 'var(--danger)';
    if (percent >= 70) return 'var(--warning)';
    return 'var(--success)';
  }
</script>

<div class="disk-panel">
  <div class="panel-header">
    <h3 class="panel-title">
      <svg class="title-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <path d="M12 2v20M2 12h20"/>
      </svg>
      Disk
      {#if $diskMetrics?.disks.length}
        <span class="disk-count">({$diskMetrics.disks.length})</span>
      {/if}
    </h3>
  </div>

  {#if $diskMetrics}
    {#if $diskMetrics.disks.length > 0}
      <div class="disks-grid">
        {#each $diskMetrics.disks as disk}
          <div class="disk-card">
            <div class="card-header">
              <span class="disk-name" title={disk.name}>{disk.name}</span>
              {#if disk.is_removable}
                <span class="removable-badge">USB</span>
              {/if}
            </div>

            <div class="disk-info">
              <div class="info-row">
                <span class="info-label">Mount</span>
                <span class="info-value" title={disk.mount_point}>{disk.mount_point}</span>
              </div>
              <div class="info-row">
                <span class="info-label">Filesystem</span>
                <span class="info-value">{disk.file_system}</span>
              </div>
            </div>

            <div class="usage-section">
              <div class="usage-bar-container">
                <div
                  class="usage-bar"
                  style:width="{disk.used_percent}%"
                  style:background-color={getUsageColor(disk.used_percent)}
                ></div>
              </div>
              <span class="usage-percent" style:color={getUsageColor(disk.used_percent)}>
                {disk.used_percent.toFixed(1)}% used
              </span>
            </div>

            <div class="disk-stats">
              <div class="stat">
                <span class="stat-label">Total</span>
                <span class="stat-value">{formatBytes(disk.total_bytes)}</span>
              </div>
              <div class="stat">
                <span class="stat-label">Used</span>
                <span class="stat-value">{formatBytes(disk.used_bytes)}</span>
              </div>
              <div class="stat">
                <span class="stat-label">Available</span>
                <span class="stat-value">{formatBytes(disk.available_bytes)}</span>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="empty-state">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <path d="M12 2v20M2 12h20"/>
        </svg>
        <p>No disks detected</p>
      </div>
    {/if}
  {:else}
    <div class="loading-state">
      <div class="spinner"></div>
      <span>Loading disk data...</span>
    </div>
  {/if}
</div>

<style>
  .disk-panel {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
    margin: var(--space-4) var(--space-6) 0;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-4);
  }

  .panel-title {
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
    color: var(--accent);
  }

  .disk-count {
    color: var(--text-muted);
    font-weight: 400;
  }

  .disks-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: var(--space-4);
  }

  .disk-card {
    background: var(--bg-elevated);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
    transition: all var(--transition-fast);
  }

  .disk-card:hover {
    border-color: var(--border-color);
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-3);
  }

  .disk-name {
    font-weight: 600;
    font-size: 0.9375rem;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 180px;
  }

  .removable-badge {
    font-size: 0.6875rem;
    padding: var(--space-1) var(--space-2);
    background: rgba(245, 158, 11, 0.2);
    color: var(--warning);
    border-radius: var(--radius-sm);
    font-weight: 500;
  }

  .disk-info {
    margin-bottom: var(--space-3);
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-1) 0;
    border-bottom: 1px solid var(--border-color);
  }

  .info-row:last-child {
    border-bottom: none;
  }

  .info-label {
    font-size: 0.75rem;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.025em;
  }

  .info-value {
    font-size: 0.8125rem;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 140px;
  }

  .usage-section {
    margin-bottom: var(--space-3);
  }

  .usage-bar-container {
    height: 8px;
    background: var(--bg-surface);
    border-radius: var(--radius-sm);
    overflow: hidden;
    margin-bottom: var(--space-1);
  }

  .usage-bar {
    height: 100%;
    border-radius: var(--radius-sm);
    transition: width var(--transition-fast), background-color var(--transition-fast);
  }

  .usage-percent {
    font-size: 0.8125rem;
    font-weight: 500;
  }

  .disk-stats {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-2);
  }

  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: var(--space-2);
    background: var(--bg-surface);
    border-radius: var(--radius-md);
  }

  .stat-label {
    font-size: 0.6875rem;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.025em;
    margin-bottom: var(--space-1);
  }

  .stat-value {
    font-size: 0.75rem;
    color: var(--text-primary);
    font-weight: 500;
  }

  .loading-state,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    color: var(--text-secondary);
    text-align: center;
  }

  .empty-state svg {
    width: 48px;
    height: 48px;
    margin-bottom: var(--space-3);
    color: var(--text-muted);
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 2px solid var(--border-color);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: var(--space-3);
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  @media (max-width: 640px) {
    .disk-panel {
      margin: var(--space-3) var(--space-4) 0;
      padding: var(--space-3);
    }

    .disks-grid {
      grid-template-columns: 1fr;
    }

    .disk-name {
      max-width: 200px;
    }

    .info-value {
      max-width: 180px;
    }

    .disk-stats {
      grid-template-columns: 1fr;
      gap: var(--space-1);
    }

    .stat {
      flex-direction: row;
      justify-content: space-between;
      text-align: left;
    }
  }
</style>

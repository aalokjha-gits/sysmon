<script lang="ts">
  import { networkMetrics } from '$lib/stores/metrics';
  import { formatBytes } from '$lib/utils/format';

  function getUsageColor(percent: number): string {
    if (percent >= 90) return 'var(--danger)';
    if (percent >= 70) return 'var(--warning)';
    return 'var(--success)';
  }
</script>

<div class="network-panel">
  <div class="panel-header">
    <h3 class="panel-title">
      <svg class="title-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="2" y="2" width="20" height="8" rx="2" ry="2"/>
        <rect x="2" y="14" width="20" height="8" rx="2" ry="2"/>
        <line x1="6" y1="6" x2="6.01" y2="6"/>
        <line x1="6" y1="18" x2="6.01" y2="18"/>
      </svg>
      Network
      {#if $networkMetrics?.interfaces.length}
        <span class="interface-count">({$networkMetrics.interfaces.length})</span>
      {/if}
    </h3>
  </div>

  {#if $networkMetrics}
    <div class="network-summary">
      <div class="summary-item">
        <span class="summary-label">Total Received</span>
        <span class="summary-value">{formatBytes($networkMetrics.total_received_bytes)}</span>
      </div>
      <div class="summary-item">
        <span class="summary-label">Total Transmitted</span>
        <span class="summary-value">{formatBytes($networkMetrics.total_transmitted_bytes)}</span>
      </div>
    </div>

    {#if $networkMetrics.interfaces.length > 0}
      <div class="interfaces-table-wrapper">
        <table class="interfaces-table">
          <thead>
            <tr>
              <th>Interface</th>
              <th class="numeric">Received</th>
              <th class="numeric">Transmitted</th>
              <th class="numeric">Packets (RX/TX)</th>
            </tr>
          </thead>
          <tbody>
            {#each $networkMetrics.interfaces.slice(0, 10) as iface}
              <tr>
                <td class="name-cell">{iface.name}</td>
                <td class="numeric-cell">{formatBytes(iface.received_bytes)}</td>
                <td class="numeric-cell">{formatBytes(iface.transmitted_bytes)}</td>
                <td class="numeric-cell">
                  {iface.received_packets.toLocaleString()} / {iface.transmitted_packets.toLocaleString()}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {:else}
      <div class="empty-state">
        <p>No network interfaces detected</p>
      </div>
    {/if}
  {:else}
    <div class="loading-state">
      <div class="spinner"></div>
      <span>Loading network data...</span>
    </div>
  {/if}
</div>

<style>
  .network-panel {
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

  .interface-count {
    color: var(--text-muted);
    font-weight: 400;
  }

  .network-summary {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-4);
    margin-bottom: var(--space-4);
    padding: var(--space-3);
    background: var(--bg-elevated);
    border-radius: var(--radius-md);
  }

  .summary-item {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .summary-label {
    font-size: 0.75rem;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.025em;
  }

  .summary-value {
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .interfaces-table-wrapper {
    overflow-x: auto;
    max-height: 300px;
    overflow-y: auto;
  }

  .interfaces-table {
    width: 100%;
    font-size: 0.8125rem;
  }

  thead {
    position: sticky;
    top: 0;
    z-index: 10;
  }

  th {
    background: var(--bg-elevated);
    color: var(--text-secondary);
    font-weight: 500;
    text-align: left;
    padding: var(--space-2) var(--space-3);
    border-bottom: 1px solid var(--border-color);
    white-space: nowrap;
  }

  th.numeric {
    text-align: right;
  }

  td {
    padding: var(--space-2) var(--space-3);
    border-bottom: 1px solid var(--border-color);
    color: var(--text-primary);
  }

  tbody tr {
    transition: background var(--transition-fast);
  }

  tbody tr:hover {
    background: var(--bg-elevated);
  }

  .name-cell {
    font-weight: 500;
  }

  .numeric-cell {
    text-align: right;
    font-family: 'SF Mono', Monaco, monospace;
    font-variant-numeric: tabular-nums;
    color: var(--text-secondary);
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
    .network-panel {
      margin: var(--space-3) var(--space-4) 0;
      padding: var(--space-3);
    }

    .network-summary {
      grid-template-columns: 1fr;
    }

    .interfaces-table {
      font-size: 0.75rem;
    }

    th, td {
      padding: var(--space-1) var(--space-2);
    }
  }
</style>

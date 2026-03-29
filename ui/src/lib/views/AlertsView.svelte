<script lang="ts">
  import { alerts, criticalAlertsCount, warningAlertsCount } from '$lib/stores/metrics';

  function getSeverityColor(severity: string): string {
    switch (severity) {
      case 'critical': return 'var(--danger)';
      case 'warning': return 'var(--warning)';
      default: return 'var(--info)';
    }
  }

  function getSeverityIcon(severity: string): string {
    switch (severity) {
      case 'critical': return '⚠';
      case 'warning': return '●';
      default: return 'ℹ';
    }
  }

  const sortedAlerts = $derived(
    [...$alerts].sort((a, b) => {
      const sev = { critical: 0, warning: 1, info: 2 };
      const sa = sev[a.severity as keyof typeof sev] ?? 3;
      const sb = sev[b.severity as keyof typeof sev] ?? 3;
      return sa - sb;
    })
  );
</script>

<div class="alerts-view">
  <div class="alerts-header">
    <h2 class="title">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="title-icon">
        <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0zM12 9v4M12 17h.01" />
      </svg>
      Active Alerts
    </h2>
    <div class="summary">
      {#if $criticalAlertsCount > 0}
        <span class="count critical">{$criticalAlertsCount} critical</span>
      {/if}
      {#if $warningAlertsCount > 0}
        <span class="count warning">{$warningAlertsCount} warning</span>
      {/if}
      {#if $alerts.length === 0}
        <span class="count clear">All clear</span>
      {/if}
    </div>
  </div>

  {#if $alerts.length === 0}
    <div class="empty-state">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="empty-icon">
        <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
        <polyline points="22 4 12 14.01 9 11.01" />
      </svg>
      <p class="empty-text">No active alerts</p>
      <p class="empty-sub">System is operating within normal thresholds</p>
    </div>
  {:else}
    <div class="alerts-list">
      {#each sortedAlerts as alert (alert.id)}
        <div class="alert-card" style:border-left-color={getSeverityColor(alert.severity)}>
          <div class="alert-header">
            <span class="severity-icon" style:color={getSeverityColor(alert.severity)}>
              {getSeverityIcon(alert.severity)}
            </span>
            <span class="alert-type">{alert.alert_type}</span>
            <span class="severity-badge" style:background={getSeverityColor(alert.severity)}>
              {alert.severity}
            </span>
          </div>
          <p class="alert-message">{alert.message}</p>
          <div class="alert-meta">
            <span class="meta-item">
              Value: <strong>{alert.value.toFixed(1)}%</strong>
            </span>
            <span class="meta-item">
              Threshold: <strong>{alert.threshold}%</strong>
            </span>
            {#if alert.since}
              <span class="meta-item">
                Since: {alert.since}
              </span>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .alerts-view {
    height: 100%;
    overflow-y: auto;
    padding: 12px;
  }

  .alerts-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
  }

  .title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: var(--font-lg);
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .title-icon {
    width: 20px;
    height: 20px;
    color: var(--warning);
  }

  .summary {
    display: flex;
    gap: 8px;
  }

  .count {
    font-size: var(--font-xs);
    font-weight: 600;
    padding: 2px 8px;
    border-radius: var(--radius-sm);
  }

  .count.critical {
    background: rgba(239, 68, 68, 0.15);
    color: var(--danger);
  }

  .count.warning {
    background: rgba(245, 158, 11, 0.15);
    color: var(--warning);
  }

  .count.clear {
    background: rgba(34, 197, 94, 0.15);
    color: var(--success);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 60px 20px;
    color: var(--text-muted);
  }

  .empty-icon {
    width: 48px;
    height: 48px;
    color: var(--success);
    margin-bottom: 16px;
    opacity: 0.6;
  }

  .empty-text {
    font-size: var(--font-lg);
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: 4px;
  }

  .empty-sub {
    font-size: var(--font-sm);
    color: var(--text-muted);
  }

  .alerts-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .alert-card {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-left: 3px solid;
    border-radius: var(--radius-md);
    padding: 12px 14px;
  }

  .alert-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }

  .severity-icon {
    font-size: 14px;
  }

  .alert-type {
    font-size: var(--font-sm);
    font-weight: 600;
    color: var(--text-primary);
    text-transform: capitalize;
  }

  .severity-badge {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    padding: 1px 6px;
    border-radius: 3px;
    color: #fff;
    margin-left: auto;
  }

  .alert-message {
    font-size: var(--font-sm);
    color: var(--text-secondary);
    margin: 0 0 8px;
    line-height: 1.4;
  }

  .alert-meta {
    display: flex;
    gap: 16px;
    font-size: var(--font-xs);
    color: var(--text-muted);
  }

  .meta-item strong {
    color: var(--text-secondary);
    font-family: var(--font-mono);
  }
</style>

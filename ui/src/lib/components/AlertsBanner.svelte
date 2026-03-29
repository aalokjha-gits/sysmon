<script lang="ts">
  import { alerts, criticalAlertsCount, warningAlertsCount } from '$lib/stores/metrics';

  let expanded = $state(false);

  function toggleExpanded() {
    expanded = !expanded;
  }

  function getSeverityColor(severity: string): string {
    switch (severity) {
      case 'critical':
        return 'var(--danger)';
      case 'warning':
        return 'var(--warning)';
      default:
        return 'var(--info)';
    }
  }

  function getSeverityBg(severity: string): string {
    switch (severity) {
      case 'critical':
        return 'rgba(239, 68, 68, 0.15)';
      case 'warning':
        return 'rgba(245, 158, 11, 0.15)';
      default:
        return 'rgba(6, 182, 212, 0.15)';
    }
  }

  const bannerColor = $derived($criticalAlertsCount > 0 ? 'var(--danger)' : $warningAlertsCount > 0 ? 'var(--warning)' : 'var(--info)');
  const bannerBg = $derived($criticalAlertsCount > 0 ? 'rgba(239, 68, 68, 0.1)' : $warningAlertsCount > 0 ? 'rgba(245, 158, 11, 0.1)' : 'rgba(6, 182, 212, 0.1)');
</script>

{#if $alerts.length > 0}
  <div class="alerts-banner" style:background-color={bannerBg} style:border-color={bannerColor}>
    <button class="banner-header" onclick={toggleExpanded} aria-expanded={expanded}>
      <div class="banner-left">
        <svg class="alert-icon" viewBox="0 0 24 24" fill="none" stroke={bannerColor} stroke-width="2">
          <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
          <line x1="12" y1="9" x2="12" y2="13" />
          <line x1="12" y1="17" x2="12.01" y2="17" />
        </svg>
        <span class="alert-count" style:color={bannerColor}>
          {$alerts.length} Active Alert{$alerts.length > 1 ? 's' : ''}
        </span>
        {#if $criticalAlertsCount > 0}
          <span class="severity-badge critical">{$criticalAlertsCount} Critical</span>
        {/if}
        {#if $warningAlertsCount > 0}
          <span class="severity-badge warning">{$warningAlertsCount} Warning</span>
        {/if}
      </div>
      <svg
        class="expand-icon"
        class:expanded={expanded}
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <polyline points="6 9 12 15 18 9" />
      </svg>
    </button>

    {#if expanded}
      <div class="alerts-list animate-slide-down">
        {#each $alerts as alert (alert.id)}
          <div class="alert-item" style:background-color={getSeverityBg(alert.severity)}>
            <div class="alert-indicator" style:background-color={getSeverityColor(alert.severity)}></div>
            <div class="alert-content">
              <div class="alert-header">
                <span class="alert-type">{alert.alert_type}</span>
                <span class="alert-severity" style:color={getSeverityColor(alert.severity)}>
                  {alert.severity}
                </span>
              </div>
              <p class="alert-message">{alert.message}</p>
              <div class="alert-meta">
                <span class="alert-value">{alert.value.toFixed(1)}% / threshold {alert.threshold}%</span>
                <span class="alert-since">since {new Date(alert.since).toLocaleTimeString()}</span>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  .alerts-banner {
    border: 1px solid;
    border-left: 4px solid;
    border-radius: var(--radius-md);
    margin: var(--space-4) var(--space-6) 0;
    overflow: hidden;
  }

  .banner-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: var(--space-3) var(--space-4);
    background: transparent;
    border: none;
    cursor: pointer;
    color: var(--text-primary);
    transition: background var(--transition-fast);
  }

  .banner-header:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .banner-left {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .alert-icon {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }

  .alert-count {
    font-weight: 600;
    font-size: 0.9375rem;
  }

  .severity-badge {
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
    font-size: 0.75rem;
    font-weight: 500;
  }

  .severity-badge.critical {
    background: rgba(239, 68, 68, 0.2);
    color: var(--danger);
  }

  .severity-badge.warning {
    background: rgba(245, 158, 11, 0.2);
    color: var(--warning);
  }

  .expand-icon {
    width: 20px;
    height: 20px;
    color: var(--text-secondary);
    transition: transform var(--transition-fast);
  }

  .expand-icon.expanded {
    transform: rotate(180deg);
  }

  .alerts-list {
    padding: 0 var(--space-4) var(--space-4);
  }

  .alert-item {
    display: flex;
    gap: var(--space-3);
    padding: var(--space-3);
    border-radius: var(--radius-md);
    margin-top: var(--space-2);
  }

  .alert-indicator {
    width: 4px;
    border-radius: var(--radius-sm);
    flex-shrink: 0;
  }

  .alert-content {
    flex: 1;
    min-width: 0;
  }

  .alert-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-1);
  }

  .alert-type {
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--text-primary);
    text-transform: uppercase;
    letter-spacing: 0.025em;
  }

  .alert-severity {
    font-size: 0.75rem;
    font-weight: 500;
    text-transform: uppercase;
  }

  .alert-message {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin: 0 0 var(--space-1) 0;
    line-height: 1.5;
  }

  .alert-meta {
    display: flex;
    gap: var(--space-4);
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  @media (max-width: 640px) {
    .alerts-banner {
      margin: var(--space-3) var(--space-4) 0;
    }

    .banner-left {
      flex-wrap: wrap;
    }

    .alert-meta {
      flex-direction: column;
      gap: var(--space-1);
    }
  }
</style>

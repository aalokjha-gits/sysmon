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

  const hasAlerts = $derived($alerts.length > 0);
  const alertColor = $derived($criticalAlertsCount > 0 ? 'var(--danger)' : $warningAlertsCount > 0 ? 'var(--warning)' : 'var(--info)');
</script>

{#if hasAlerts}
  <div class="alerts-compact" style:border-color={alertColor}>
    <button class="alert-toggle" onclick={toggleExpanded} aria-expanded={expanded}>
      <span class="alert-icon" style:color={alertColor}>
        {#if $criticalAlertsCount > 0}
          ⚠
        {:else}
          ●
        {/if}
      </span>
      <span class="alert-text">
        {$alerts.length} alert{$alerts.length > 1 ? 's' : ''}:
        <span class="alert-summary">{$alerts[0].message}</span>
        {#if $alerts.length > 1}
          <span class="alert-more">+{$alerts.length - 1} more</span>
        {/if}
      </span>
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
      <div class="alerts-list">
        {#each $alerts as alert (alert.id)}
          <div class="alert-item">
            <span class="alert-dot" style:background-color={getSeverityColor(alert.severity)}></span>
            <span class="alert-type">{alert.alert_type}</span>
            <span class="alert-msg">{alert.message}</span>
            <span class="alert-meta">{alert.value.toFixed(1)}% / {alert.threshold}%</span>
          </div>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  .alerts-compact {
    border-left: 3px solid;
    background: var(--bg-elevated);
    margin: 2px 0;
  }

  .alert-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 12px;
    background: transparent;
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    font-size: var(--font-sm);
    text-align: left;
  }

  .alert-toggle:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  .alert-icon {
    font-size: 12px;
    flex-shrink: 0;
  }

  .alert-text {
    flex: 1;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .alert-summary {
    color: var(--text-secondary);
  }

  .alert-more {
    color: var(--text-muted);
    margin-left: 4px;
  }

  .expand-icon {
    width: 14px;
    height: 14px;
    color: var(--text-muted);
    flex-shrink: 0;
    transition: transform var(--transition-fast);
  }

  .expand-icon.expanded {
    transform: rotate(180deg);
  }

  .alerts-list {
    padding: 0 12px 8px 32px;
  }

  .alert-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 0;
    font-size: var(--font-xs);
    border-bottom: 1px solid var(--border-color);
  }

  .alert-item:last-child {
    border-bottom: none;
  }

  .alert-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .alert-type {
    color: var(--text-secondary);
    text-transform: uppercase;
    font-weight: 500;
    min-width: 60px;
  }

  .alert-msg {
    flex: 1;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .alert-meta {
    color: var(--text-muted);
    font-family: var(--font-mono);
  }
</style>

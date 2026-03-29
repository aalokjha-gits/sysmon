<script lang="ts">
  import { connected, serverPort, serverUptime, reconnect, connecting, connectionError } from '$lib/stores/metrics';
  import { formatUptime } from '$lib/utils/format';

  const VERSION = 'v0.1.0';

  function handleReconnect() {
    reconnect();
  }
</script>

<header class="header">
  <div class="header-left">
    <div class="logo">
      <svg class="logo-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="2" y="3" width="20" height="14" rx="2" />
        <line x1="8" y1="21" x2="16" y2="21" />
        <line x1="12" y1="17" x2="12" y2="21" />
        <path d="M6 8h.01M6 12h.01M10 8h.01M10 12h.01M14 8h.01M14 12h.01M18 8h.01M18 12h.01" />
      </svg>
      <h1 class="title">sysmon <span class="version">{VERSION}</span></h1>
    </div>
  </div>

  <div class="header-right">
    {#if $connectionError}
      <button class="status-badge error" onclick={handleReconnect} title="Click to reconnect">
        <span class="status-dot error"></span>
        <span class="status-text">Error</span>
      </button>
    {:else if $connecting}
      <div class="status-badge connecting">
        <span class="status-dot connecting"></span>
        <span class="status-text">Connecting...</span>
      </div>
    {:else if $connected}
      <div class="status-badge connected">
        <span class="status-dot connected"></span>
        <span class="status-text">Connected</span>
      </div>
    {:else}
      <button class="status-badge disconnected" onclick={handleReconnect} title="Click to reconnect">
        <span class="status-dot disconnected"></span>
        <span class="status-text">Disconnected</span>
      </button>
    {/if}

    {#if $serverPort}
      <div class="port-badge" title="Server port">
        <svg class="port-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10" />
          <line x1="2" y1="12" x2="22" y2="12" />
          <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
        </svg>
        <span class="port-text">:{$serverPort}</span>
      </div>
    {/if}

    {#if $serverUptime > 0}
      <div class="uptime-badge" title="Server uptime">
        <svg class="uptime-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10" />
          <polyline points="12 6 12 12 16 14" />
        </svg>
        <span class="uptime-text">{formatUptime($serverUptime)}</span>
      </div>
    {/if}
  </div>
</header>

<style>
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
  }

  .header-left {
    display: flex;
    align-items: center;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .logo-icon {
    width: 20px;
    height: 20px;
    color: var(--accent);
  }

  .title {
    font-size: var(--font-base);
    font-weight: 600;
    margin: 0;
    color: var(--text-primary);
    display: flex;
    align-items: baseline;
    gap: 6px;
  }

  .version {
    font-size: var(--font-xs);
    font-weight: 500;
    color: var(--text-muted);
    background: var(--bg-elevated);
    padding: 1px 5px;
    border-radius: var(--radius-sm);
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .status-badge,
  .port-badge,
  .uptime-badge {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 3px 8px;
    border-radius: var(--radius-sm);
    font-size: var(--font-xs);
    font-weight: 500;
  }

  .status-badge {
    background: var(--bg-elevated);
    border: 1px solid var(--border-color);
    transition: all var(--transition-fast);
  }

  .status-badge:hover {
    background: var(--bg-surface-hover);
  }

  .status-badge.connected {
    background: rgba(34, 197, 94, 0.1);
    border-color: rgba(34, 197, 94, 0.3);
  }

  .status-badge.disconnected {
    background: rgba(239, 68, 68, 0.1);
    border-color: rgba(239, 68, 68, 0.3);
    cursor: pointer;
  }

  .status-badge.connecting {
    background: rgba(245, 158, 11, 0.1);
    border-color: rgba(245, 158, 11, 0.3);
  }

  .status-badge.error {
    background: rgba(239, 68, 68, 0.15);
    border-color: rgba(239, 68, 68, 0.4);
    cursor: pointer;
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }

  .status-dot.connected {
    background: var(--success);
    box-shadow: 0 0 6px var(--success);
  }

  .status-dot.disconnected {
    background: var(--danger);
  }

  .status-dot.connecting {
    background: var(--warning);
    animation: pulse 1.5s infinite;
  }

  .status-dot.error {
    background: var(--danger);
    animation: pulse 1s infinite;
  }

  .port-badge,
  .uptime-badge {
    background: var(--bg-elevated);
    color: var(--text-secondary);
    border: 1px solid var(--border-color);
  }

  .port-icon,
  .uptime-icon {
    width: 12px;
    height: 12px;
  }

  @media (max-width: 640px) {
    .header {
      padding: 6px 10px;
      flex-wrap: wrap;
      gap: 6px;
    }

    .title {
      font-size: var(--font-sm);
    }

    .version {
      display: none;
    }

    .uptime-badge {
      display: none;
    }

    .port-badge {
      display: none;
    }
  }
</style>

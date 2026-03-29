<script lang="ts">
  import { metrics, serverPort, serverUptime } from '$lib/stores/metrics';
  import LoadCompact from '$lib/components/LoadCompact.svelte';
  import DiskCompact from '$lib/components/DiskCompact.svelte';
  import { formatUptime } from '$lib/utils/format';

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return (bytes / Math.pow(k, i)).toFixed(1) + ' ' + sizes[i];
  }

  const system = $derived($metrics?.system ?? null);
</script>

<div class="system-view">
  <div class="section info-section">
    <h2 class="section-title">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="section-icon">
        <rect x="2" y="3" width="20" height="14" rx="2" />
        <line x1="8" y1="21" x2="16" y2="21" />
        <line x1="12" y1="17" x2="12" y2="21" />
      </svg>
      System Information
    </h2>
    <div class="info-grid">
      {#if system}
        <div class="info-item">
          <span class="info-label">Hostname</span>
          <span class="info-value">{system.hostname ?? 'Unknown'}</span>
        </div>
        <div class="info-item">
          <span class="info-label">OS</span>
          <span class="info-value">{system.os_name ?? 'Unknown'} {system.os_version ?? ''}</span>
        </div>
        <div class="info-item">
          <span class="info-label">Kernel</span>
          <span class="info-value">{system.kernel_version ?? 'Unknown'}</span>
        </div>
        <div class="info-item">
          <span class="info-label">CPU Cores</span>
          <span class="info-value">{system.cpu_count ?? 'Unknown'}</span>
        </div>
        <div class="info-item">
          <span class="info-label">Total Memory</span>
          <span class="info-value">{system.total_memory_bytes ? formatBytes(system.total_memory_bytes) : 'Unknown'}</span>
        </div>
        <div class="info-item">
          <span class="info-label">Uptime</span>
          <span class="info-value">{system.uptime_seconds ? formatUptime(system.uptime_seconds) : 'Unknown'}</span>
        </div>
        {#if $serverPort}
          <div class="info-item">
            <span class="info-label">Server Port</span>
            <span class="info-value mono">:{$serverPort}</span>
          </div>
        {/if}
        {#if $serverUptime > 0}
          <div class="info-item">
            <span class="info-label">Server Uptime</span>
            <span class="info-value">{formatUptime($serverUptime)}</span>
          </div>
        {/if}
      {:else}
        <div class="loading">Waiting for data...</div>
      {/if}
    </div>
  </div>

  <div class="two-col">
    <div class="section">
      <h2 class="section-title">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="section-icon">
          <path d="M22 12h-4l-3 9L9 3l-3 9H2" />
        </svg>
        Load Average
      </h2>
      <div class="section-content">
        <LoadCompact />
      </div>
    </div>

    <div class="section">
      <h2 class="section-title">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="section-icon">
          <ellipse cx="12" cy="5" rx="9" ry="3" />
          <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3" />
          <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5" />
        </svg>
        Disk Usage
      </h2>
      <div class="section-content">
        <DiskCompact />
      </div>
    </div>
  </div>
</div>

<style>
  .system-view {
    height: 100%;
    overflow-y: auto;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .section {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .section-title {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    font-size: var(--font-sm);
    font-weight: 600;
    color: var(--text-primary);
    border-bottom: 1px solid var(--border-color);
    margin: 0;
  }

  .section-icon {
    width: 16px;
    height: 16px;
    color: var(--accent);
    flex-shrink: 0;
  }

  .section-content {
    padding: 0;
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 1px;
    background: var(--border-color);
    padding: 0;
  }

  .info-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 10px 14px;
    background: var(--bg-surface);
  }

  .info-label {
    font-size: var(--font-xs);
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    font-weight: 500;
  }

  .info-value {
    font-size: var(--font-sm);
    color: var(--text-primary);
    font-weight: 500;
  }

  .info-value.mono {
    font-family: var(--font-mono);
  }

  .loading {
    padding: 20px;
    color: var(--text-muted);
    font-size: var(--font-sm);
    text-align: center;
    background: var(--bg-surface);
  }

  .two-col {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  @media (max-width: 768px) {
    .two-col {
      grid-template-columns: 1fr;
    }
  }
</style>

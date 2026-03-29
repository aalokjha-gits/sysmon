<script lang="ts">
  import { portMetrics } from '$lib/stores/metrics';
  import type { PortInfo } from '$lib/types';

  function sortPorts(ports: PortInfo[]): PortInfo[] {
    return [...ports].sort((a, b) => {
      if (a.is_external !== b.is_external) return a.is_external ? -1 : 1;
      return a.port - b.port;
    });
  }
</script>

<div class="ports-compact compact-panel">
  <div class="panel-header">
    <span class="compact-panel-title">Ports</span>
    {#if $portMetrics.length > 0}
      <span class="port-count">
        {$portMetrics.length}
        {#if $portMetrics.filter(p => p.is_external).length > 0}
          <span class="external-badge">{$portMetrics.filter(p => p.is_external).length} exposed</span>
        {/if}
      </span>
    {/if}
  </div>

  {#if $portMetrics.length > 0}
    <div class="ports-list">
      {#each sortPorts($portMetrics) as port}
        <div class="port-row" class:external={port.is_external}>
          <span class="port-number mono">{port.port}</span>
          <span class="port-proto mono">{port.protocol}</span>
          <span class="port-process">{port.process_name}</span>
          <span class="port-address mono">{port.address}</span>
          {#if port.service}
            <span class="port-service">{port.service}</span>
          {/if}
          {#if port.is_external}
            <span class="external-icon" title="Exposed to network">⚠</span>
          {/if}
        </div>
      {/each}
    </div>
  {:else}
    <div class="ports-list">
      <div class="port-row skeleton">
        <span class="port-number mono">--</span>
        <span class="port-proto mono">--</span>
        <span class="port-process">no listeners</span>
      </div>
    </div>
  {/if}
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

  .port-count {
    font-size: var(--font-xs);
    color: var(--text-muted);
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .external-badge {
    font-size: 0.625rem;
    color: var(--warning);
    background: rgba(245, 158, 11, 0.1);
    padding: 1px 5px;
    border-radius: 3px;
    font-weight: 500;
  }

  .ports-list {
    display: flex;
    flex-direction: column;
    gap: 3px;
    max-height: 180px;
    overflow-y: auto;
  }

  .ports-list::-webkit-scrollbar {
    width: 4px;
  }

  .ports-list::-webkit-scrollbar-track {
    background: transparent;
  }

  .ports-list::-webkit-scrollbar-thumb {
    background: var(--border-color);
    border-radius: 2px;
  }

  .port-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: var(--font-xs);
    padding: 2px 0;
    overflow: hidden;
  }

  .port-row.skeleton {
    opacity: 0.5;
  }

  .port-row.external {
    color: var(--warning);
  }

  .port-number {
    min-width: 40px;
    color: var(--accent);
    font-weight: 600;
  }

  .port-row.external .port-number {
    color: var(--warning);
  }

  .port-proto {
    min-width: 28px;
    color: var(--text-muted);
    font-size: 0.625rem;
    text-transform: uppercase;
  }

  .port-process {
    flex: 1;
    color: var(--text-primary);
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .port-address {
    color: var(--text-muted);
    font-size: 0.625rem;
    white-space: nowrap;
  }

  .port-service {
    font-size: 0.625rem;
    color: var(--text-secondary);
    background: var(--bg-primary);
    padding: 0 4px;
    border-radius: 2px;
    white-space: nowrap;
  }

  .external-icon {
    font-size: 0.7rem;
    flex-shrink: 0;
  }

  .mono {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
</style>

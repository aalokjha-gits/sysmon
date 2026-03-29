<script lang="ts">
  import { networkMetrics } from '$lib/stores/metrics';
  import { formatBytes } from '$lib/utils/format';

  function formatPackets(packets: number): string {
    if (packets >= 1000000) {
      return (packets / 1000000).toFixed(1) + 'M';
    }
    if (packets >= 1000) {
      return (packets / 1000).toFixed(1) + 'K';
    }
    return packets.toString();
  }

  // Sort interfaces by total traffic (received + transmitted)
  function getTopInterfaces(interfaces: NonNullable<typeof $networkMetrics>['interfaces'], limit: number = 5) {
    return [...interfaces]
      .sort((a, b) => (b.received_bytes + b.transmitted_bytes) - (a.received_bytes + a.transmitted_bytes))
      .slice(0, limit);
  }
</script>

<div class="network-compact compact-panel">
  <div class="panel-header">
    <span class="compact-panel-title">Network</span>
  </div>

  {#if $networkMetrics}
    <div class="interfaces-list">
      {#each getTopInterfaces($networkMetrics.interfaces) as iface}
        <div class="interface-row">
          <span class="iface-name mono">{iface.name}</span>
          <span class="iface-traffic">
            <span class="traffic-arrow down">↓</span>
            <span class="mono">{formatBytes(iface.received_bytes)}</span>
            <span class="traffic-arrow up">↑</span>
            <span class="mono">{formatBytes(iface.transmitted_bytes)}</span>
          </span>
          <span class="iface-packets mono">
            {formatPackets(iface.received_packets)}/{formatPackets(iface.transmitted_packets)}
          </span>
        </div>
      {/each}
    </div>
  {:else}
    <div class="interfaces-list">
      <div class="interface-row skeleton">
        <span class="iface-name mono">--</span>
        <span class="iface-traffic">
          <span class="traffic-arrow down">↓</span>
          <span class="mono">--</span>
          <span class="traffic-arrow up">↑</span>
          <span class="mono">--</span>
        </span>
        <span class="iface-packets mono">--/--</span>
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

  .interfaces-list {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .interface-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: var(--font-xs);
    padding: 2px 0;
    overflow: hidden;
  }

  .interface-row.skeleton {
    opacity: 0.5;
  }

  .iface-name {
    min-width: 50px;
    color: var(--text-primary);
    font-weight: 500;
  }

  .iface-traffic {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 2px;
    flex-shrink: 1;
    overflow: hidden;
  }

  .traffic-arrow {
    font-size: 10px;
    color: var(--text-muted);
  }

  .traffic-arrow.down {
    color: var(--success);
  }

  .traffic-arrow.up {
    color: var(--accent);
  }

  .iface-packets {
    color: var(--text-muted);
    min-width: auto;
    text-align: right;
    white-space: nowrap;
  }

  .mono {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
</style>

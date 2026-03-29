<script lang="ts">
  import { diskMetrics } from '$lib/stores/metrics';
  import { formatBytes } from '$lib/utils/format';

  function getUsageColor(percent: number): string {
    if (percent >= 90) return 'var(--danger)';
    if (percent >= 70) return 'var(--warning)';
    return 'var(--success)';
  }
</script>

<div class="disk-compact compact-panel">
  <div class="panel-header">
    <span class="compact-panel-title">Disk ({$diskMetrics?.disks.length ?? 0})</span>
  </div>

  {#if $diskMetrics}
    <div class="disks-list">
      {#each $diskMetrics.disks as disk}
        <div class="disk-row">
          <span class="disk-mount mono" title={disk.mount_point}>
            {disk.mount_point}
          </span>
          <div class="disk-bar-bg">
            <div
              class="disk-bar-fill"
              style:width="{disk.used_percent}%"
              style:background-color={getUsageColor(disk.used_percent)}
            ></div>
          </div>
          <span class="disk-percent mono" style:color={getUsageColor(disk.used_percent)}>
            {disk.used_percent.toFixed(1)}%
          </span>
          <span class="disk-size mono">
            {formatBytes(disk.used_bytes)}/{formatBytes(disk.total_bytes)}
          </span>
          <span class="disk-fs">{disk.file_system}</span>
        </div>
      {/each}
    </div>
  {:else}
    <div class="disks-list">
      <div class="disk-row skeleton">
        <span class="disk-mount mono">/</span>
        <div class="disk-bar-bg"><div class="disk-bar-fill" style:width="0%"></div></div>
        <span class="disk-percent mono">--</span>
        <span class="disk-size mono">--/--</span>
        <span class="disk-fs">--</span>
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

  .disks-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .disk-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: var(--font-xs);
    padding: 3px 0;
  }

  .disk-row.skeleton {
    opacity: 0.5;
  }

  .disk-mount {
    min-width: 50px;
    color: var(--text-primary);
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .disk-bar-bg {
    flex: 1;
    height: 4px;
    background: var(--bg-elevated);
    border-radius: 2px;
    overflow: hidden;
    min-width: 40px;
    max-width: 80px;
  }

  .disk-bar-fill {
    height: 100%;
    border-radius: 2px;
    transition: width 300ms ease;
  }

  .disk-percent {
    min-width: 36px;
    text-align: right;
    font-weight: 500;
  }

  .disk-size {
    color: var(--text-secondary);
    min-width: 80px;
    text-align: right;
  }

  .disk-fs {
    color: var(--text-muted);
    min-width: 30px;
    text-align: right;
  }

  .mono {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
</style>

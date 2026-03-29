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
        <div class="disk-entry">
          <div class="disk-top-line">
            <span class="disk-mount mono" title={disk.mount_point}>
              {disk.mount_point}
            </span>
            <span class="disk-meta">
              <span class="disk-percent mono" style:color={getUsageColor(disk.used_percent)}>
                {disk.used_percent.toFixed(1)}%
              </span>
              <span class="disk-size mono">
                {formatBytes(disk.used_bytes)}/{formatBytes(disk.total_bytes)}
              </span>
              <span class="disk-fs">{disk.file_system}</span>
            </span>
          </div>
          <div class="disk-bar-bg">
            <div
              class="disk-bar-fill"
              style:width="{disk.used_percent}%"
              style:background-color={getUsageColor(disk.used_percent)}
            ></div>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="disks-list">
      <div class="disk-entry skeleton">
        <div class="disk-top-line">
          <span class="disk-mount mono">/</span>
          <span class="disk-meta">
            <span class="disk-percent mono">--</span>
            <span class="disk-size mono">--/--</span>
          </span>
        </div>
        <div class="disk-bar-bg"><div class="disk-bar-fill" style:width="0%"></div></div>
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

  .disks-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .disk-entry {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .disk-entry.skeleton {
    opacity: 0.5;
  }

  .disk-top-line {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
    font-size: var(--font-xs);
  }

  .disk-mount {
    color: var(--text-primary);
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
    flex-shrink: 1;
  }

  .disk-meta {
    display: flex;
    align-items: baseline;
    gap: 6px;
    flex-shrink: 0;
    white-space: nowrap;
  }

  .disk-bar-bg {
    width: 100%;
    height: 4px;
    background: var(--bg-elevated);
    border-radius: 2px;
    overflow: hidden;
  }

  .disk-bar-fill {
    height: 100%;
    border-radius: 2px;
    transition: width 300ms ease;
  }

  .disk-percent {
    font-weight: 500;
  }

  .disk-size {
    color: var(--text-secondary);
  }

  .disk-fs {
    color: var(--text-muted);
  }

  .mono {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
</style>

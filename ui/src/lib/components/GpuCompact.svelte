<script lang="ts">
  import { gpuMetrics } from '$lib/stores/metrics';
  import { formatBytes } from '$lib/utils/format';

  function getUsageColor(percent: number | null): string {
    if (percent === null) return 'var(--text-muted)';
    if (percent >= 90) return 'var(--danger)';
    if (percent >= 70) return 'var(--warning)';
    return 'var(--success)';
  }

  function getVramPercent(used: number | null, total: number | null): number {
    if (used === null || total === null || total === 0) return 0;
    return Math.min(100, Math.max(0, (used / total) * 100));
  }
</script>

<div class="gpu-compact compact-panel">
  <div class="panel-header">
    <span class="compact-panel-title">GPU ({$gpuMetrics?.gpus.length ?? 0})</span>
  </div>

  {#if $gpuMetrics && $gpuMetrics.gpus.length > 0}
    <div class="gpus-list">
      {#each $gpuMetrics.gpus as gpu}
        <div class="gpu-entry">
          <div class="gpu-top-line">
            <span class="gpu-name mono" title="{gpu.vendor} {gpu.name}">
              {gpu.name}
            </span>
            <span class="gpu-meta">
              {#if gpu.utilization_percent !== null}
                <span class="gpu-util mono" style:color={getUsageColor(gpu.utilization_percent)}>
                  {gpu.utilization_percent.toFixed(1)}%
                </span>
              {:else}
                <span class="gpu-util mono" style="color: var(--text-muted)">--%</span>
              {/if}
            </span>
          </div>
          
          <div class="gpu-vram-line">
            <span class="gpu-vram-label">VRAM</span>
            <span class="gpu-vram-val mono">
              {gpu.vram_used_bytes !== null ? formatBytes(gpu.vram_used_bytes) : '--'}/{gpu.vram_total_bytes !== null ? formatBytes(gpu.vram_total_bytes) : '--'}
            </span>
          </div>
          <div class="gpu-bar-bg">
            <div
              class="gpu-bar-fill"
              style:width="{getVramPercent(gpu.vram_used_bytes, gpu.vram_total_bytes)}%"
              style:background-color={getUsageColor(getVramPercent(gpu.vram_used_bytes, gpu.vram_total_bytes))}
            ></div>
          </div>

          <div class="gpu-stats-line">
            {#if gpu.temperature_celsius !== null}
              <span class="gpu-stat mono">{gpu.temperature_celsius.toFixed(1)}°C</span>
            {/if}
            {#if gpu.power_watts !== null}
              <span class="gpu-stat mono">{gpu.power_watts.toFixed(1)}W</span>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="gpus-list empty">
      <div class="gpu-entry skeleton">
        <div class="gpu-top-line">
          <span class="gpu-name mono">No GPU detected</span>
          <span class="gpu-meta">
            <span class="gpu-util mono">--%</span>
          </span>
        </div>
        <div class="gpu-vram-line">
          <span class="gpu-vram-label">VRAM</span>
          <span class="gpu-vram-val mono">--/--</span>
        </div>
        <div class="gpu-bar-bg"><div class="gpu-bar-fill" style:width="0%"></div></div>
      </div>
    </div>
  {/if}
</div>

<style>
  .compact-panel {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md, 6px);
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

  .gpus-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .gpu-entry {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .gpu-entry.skeleton {
    opacity: 0.5;
  }

  .gpu-top-line {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
    font-size: var(--font-xs);
  }

  .gpu-name {
    color: var(--text-primary);
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
    flex-shrink: 1;
  }

  .gpu-meta {
    display: flex;
    align-items: baseline;
    gap: 6px;
    flex-shrink: 0;
    white-space: nowrap;
  }

  .gpu-util {
    font-weight: 500;
  }

  .gpu-vram-line {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    font-size: 0.75rem;
  }

  .gpu-vram-label {
    color: var(--text-muted);
  }

  .gpu-vram-val {
    color: var(--text-secondary);
  }

  .gpu-bar-bg {
    width: 100%;
    height: 4px;
    background: var(--bg-elevated);
    border-radius: 2px;
    overflow: hidden;
  }

  .gpu-bar-fill {
    height: 100%;
    border-radius: 2px;
    transition: width 300ms ease;
  }

  .gpu-stats-line {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.75rem;
    color: var(--text-secondary);
    margin-top: 2px;
  }

  .mono {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
</style>

<script lang="ts">
  import { temperatureMetrics } from '$lib/stores/metrics';

  function getTempColor(celsius: number | null): string {
    if (celsius === null) return 'var(--text-muted)';
    if (celsius >= 80) return 'var(--danger)';
    if (celsius >= 60) return 'var(--warning)';
    return 'var(--success)';
  }

  function getTempPercent(celsius: number | null, max: number | null): number {
    if (celsius === null) return 0;
    const maxVal = max ?? 100;
    // Cap at 100%
    return Math.min(100, Math.max(0, (celsius / maxVal) * 100));
  }
</script>

<div class="temp-compact compact-panel">
  <div class="panel-header">
    <span class="compact-panel-title">Temperature</span>
  </div>

  {#if $temperatureMetrics && $temperatureMetrics.sensors.length > 0}
    <div class="sensors-list">
      {#each $temperatureMetrics.sensors as sensor}
        <div class="sensor-entry">
          <div class="sensor-top-line">
            <span class="sensor-label mono" title={sensor.label}>
              {sensor.label}
            </span>
            <span class="sensor-meta">
              {#if sensor.temperature_celsius !== null}
                <span class="sensor-val mono" style:color={getTempColor(sensor.temperature_celsius)}>
                  {sensor.temperature_celsius.toFixed(1)}°C
                </span>
              {:else}
                <span class="sensor-val mono" style="color: var(--text-muted)">--°C</span>
              {/if}
              {#if sensor.max_celsius}
                <span class="sensor-max mono">max {sensor.max_celsius}°C</span>
              {/if}
            </span>
          </div>
          <div class="sensor-bar-bg">
            <div
              class="sensor-bar-fill"
              style:width="{getTempPercent(sensor.temperature_celsius, sensor.max_celsius)}%"
              style:background-color={getTempColor(sensor.temperature_celsius)}
            ></div>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="sensors-list empty">
      <div class="sensor-entry skeleton">
        <div class="sensor-top-line">
          <span class="sensor-label mono">No sensors</span>
          <span class="sensor-meta">
            <span class="sensor-val mono">--°C</span>
          </span>
        </div>
        <div class="sensor-bar-bg"><div class="sensor-bar-fill" style:width="0%"></div></div>
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

  .sensors-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .sensor-entry {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .sensor-entry.skeleton {
    opacity: 0.5;
  }

  .sensor-top-line {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
    font-size: var(--font-xs);
  }

  .sensor-label {
    color: var(--text-primary);
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
    flex-shrink: 1;
  }

  .sensor-meta {
    display: flex;
    align-items: baseline;
    gap: 6px;
    flex-shrink: 0;
    white-space: nowrap;
  }

  .sensor-bar-bg {
    width: 100%;
    height: 4px;
    background: var(--bg-elevated);
    border-radius: 2px;
    overflow: hidden;
  }

  .sensor-bar-fill {
    height: 100%;
    border-radius: 2px;
    transition: width 300ms ease;
  }

  .sensor-val {
    font-weight: 500;
  }

  .sensor-max {
    color: var(--text-secondary);
    font-size: 0.75rem;
  }

  .mono {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
</style>

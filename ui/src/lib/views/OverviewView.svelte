<script lang="ts">
  import CpuCoresChart from '$lib/components/CpuCoresChart.svelte';
  import LoadCompact from '$lib/components/LoadCompact.svelte';
  import DiskCompact from '$lib/components/DiskCompact.svelte';
  import TemperatureCompact from '$lib/components/TemperatureCompact.svelte';
  import GpuCompact from '$lib/components/GpuCompact.svelte';
  import AlertsCompact from '$lib/components/AlertsCompact.svelte';
  import TimeSeriesChart from '$lib/components/TimeSeriesChart.svelte';
  import { metrics, serverPort, serverUptime, networkMetrics, portMetrics, temperatureMetrics, gpuMetrics } from '$lib/stores/metrics';
  import { cpuHistory, memoryHistory, coreHistory, temperatureHistory, gpuHistory } from '$lib/stores/metricsHistory';
  import { setView } from '$lib/stores/navigation';
  import { formatUptime } from '$lib/utils/format';

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return (bytes / Math.pow(k, i)).toFixed(1) + ' ' + sizes[i];
  }

  const system = $derived($metrics?.system ?? null);
  const interfaceCount = $derived($networkMetrics?.interfaces?.length ?? 0);
  const portCount = $derived($portMetrics?.length ?? 0);
  const externalPorts = $derived($portMetrics?.filter(p => p.is_external)?.length ?? 0);
  const containerCount = $derived($metrics?.system?.container_count ?? 0);
</script>

<div class="overview">
  <AlertsCompact />

  <div class="system-strip">
    {#if system}
      <span class="sys-item">{system.hostname ?? 'Unknown'}</span>
      <span class="sys-sep">·</span>
      <span class="sys-item">{system.os_name ?? ''} {system.os_version ?? ''}</span>
      <span class="sys-sep">·</span>
      <span class="sys-item">{system.cpu_count ?? '?'} cores</span>
      <span class="sys-sep">·</span>
      <span class="sys-item">{system.total_memory_bytes ? formatBytes(system.total_memory_bytes) : '?'} RAM</span>
      {#if system.uptime_seconds}
        <span class="sys-sep">·</span>
        <span class="sys-item">up {formatUptime(system.uptime_seconds)}</span>
      {/if}
      {#if $serverPort}
        <span class="sys-sep">·</span>
        <span class="sys-item mono">:{$serverPort}</span>
      {/if}
    {:else}
      <span class="sys-item muted">Waiting for data...</span>
    {/if}
  </div>

  <div class="charts-row">
    <div class="chart-card">
      <TimeSeriesChart data={$cpuHistory} label="CPU" metric="cpu" unit="%" color="var(--cpu-color)" />
    </div>
    <div class="chart-card">
      <TimeSeriesChart data={$memoryHistory} label="Memory" metric="memory" unit="%" color="var(--memory-color)" />
    </div>
  </div>

  <div class="chart-card cores-chart">
    <CpuCoresChart coreData={$coreHistory} />
  </div>

  <div class="charts-row">
    <div class="chart-card">
      <TimeSeriesChart data={$temperatureHistory} label="Temp" metric="temperature" unit="°C" color="#f59e0b" minValue={0} maxValue={110} />
    </div>
    <div class="chart-card">
      <TimeSeriesChart data={$gpuHistory} label="GPU" metric="gpu_utilization" unit="%" color="#8b5cf6" />
    </div>
  </div>

  <div class="bottom-row">
    <div class="card">
      <LoadCompact />
    </div>
    <div class="card">
      <DiskCompact />
    </div>
  </div>

  <div class="bottom-row equal">
    <div class="card">
      <TemperatureCompact />
    </div>
    <div class="card">
      <GpuCompact />
    </div>
  </div>

  <div class="nav-chips">
    <button class="chip" onclick={() => setView('network')}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="chip-icon">
        <circle cx="12" cy="12" r="10" />
        <line x1="2" y1="12" x2="22" y2="12" />
      </svg>
      <span class="chip-label">{interfaceCount} interfaces</span>
      <span class="chip-sep">·</span>
      <span class="chip-label">{portCount} ports</span>
      {#if externalPorts > 0}
        <span class="chip-warn">⚠ {externalPorts} exposed</span>
      {/if}
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="chip-arrow">
        <polyline points="9 18 15 12 9 6" />
      </svg>
    </button>

    <button class="chip" onclick={() => setView('containers')}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="chip-icon">
        <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
      </svg>
      <span class="chip-label">{containerCount} containers</span>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="chip-arrow">
        <polyline points="9 18 15 12 9 6" />
      </svg>
    </button>
  </div>
</div>

<style>
  .overview {
    height: 100%;
    overflow-y: auto;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .system-strip {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    font-size: var(--font-xs);
    color: var(--text-secondary);
    flex-wrap: wrap;
    flex-shrink: 0;
  }

  .sys-sep {
    color: var(--text-muted);
  }

  .sys-item.mono {
    font-family: var(--font-mono);
  }

  .sys-item.muted {
    color: var(--text-muted);
  }

  .charts-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
    flex-shrink: 0;
  }

  .chart-card {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    padding: 10px 12px 6px;
    overflow: hidden;
    flex-shrink: 0;
  }

  .cores-chart {
    padding: 0;
  }

  .cores-chart :global(> *) {
    border: none;
    border-radius: 0;
  }

  .bottom-row {
    display: grid;
    grid-template-columns: 2fr 3fr;
    gap: 8px;
    flex-shrink: 0;
    align-items: start;
  }

  .bottom-row.equal {
    grid-template-columns: 1fr 1fr;
  }

  .card {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    overflow: hidden;
    min-width: 0;
  }

  .card :global(> *) {
    border: none;
    border-radius: 0;
  }

  .nav-chips {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
  }

  .chip {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    font-size: var(--font-xs);
    cursor: pointer;
    transition: border-color var(--transition-fast);
  }

  .chip:hover {
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .chip-icon {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    color: var(--accent);
  }

  .chip-sep {
    color: var(--text-muted);
  }

  .chip-warn {
    color: var(--warning);
    font-weight: 600;
  }

  .chip-arrow {
    width: 14px;
    height: 14px;
    margin-left: auto;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  @media (max-width: 768px) {
    .charts-row {
      grid-template-columns: 1fr;
    }

    .bottom-row,
    .bottom-row.equal {
      grid-template-columns: 1fr;
    }

    .nav-chips {
      flex-direction: column;
    }
  }
</style>

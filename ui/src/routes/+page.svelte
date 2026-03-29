<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { connect, disconnect, fetchHealth, connected, metrics, temperatureMetrics, gpuMetrics, startProcessPolling, stopProcessPolling } from '$lib/stores/metrics';
  import { startMetricsHistory, stopMetricsHistory } from '$lib/stores/metricsHistory';
  import { activeView } from '$lib/stores/navigation';
  import Header from '$lib/components/Header.svelte';
  import MetricPill from '$lib/components/MetricPill.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import OverviewView from '$lib/views/OverviewView.svelte';
  import ProcessesView from '$lib/views/ProcessesView.svelte';
  import NetworkView from '$lib/views/NetworkView.svelte';
  import ContainersView from '$lib/views/ContainersView.svelte';
  import AlertsView from '$lib/views/AlertsView.svelte';

  let healthCheckInterval: ReturnType<typeof setInterval>;

  // Derived values for metric pills
  const cpuPercent = $derived($metrics?.cpu.overall_percent ?? 0);
  const memPercent = $derived($metrics?.memory.used_percent ?? 0);
  const loadAvg1m = $derived($metrics?.load_avg[0] ?? 0);
  const netRx = $derived($metrics?.network.total_received_bytes ?? 0);
  const netTx = $derived($metrics?.network.total_transmitted_bytes ?? 0);
  const diskPercent = $derived($metrics?.disk.disks[0]?.used_percent ?? 0);
  const containerCount = $derived($metrics?.system?.container_count ?? 0);
  const portCount = $derived($metrics?.ports?.length ?? 0);
  const tempSensors = $derived($temperatureMetrics?.sensors?.filter(s => s.temperature_celsius !== null) ?? []);
  const avgTemp = $derived(tempSensors.length > 0 ? tempSensors.reduce((sum, s) => sum + (s.temperature_celsius ?? 0), 0) / tempSensors.length : 0);
  const gpuUtil = $derived($gpuMetrics?.gpus[0]?.utilization_percent ?? 0);

  onMount(() => {
    connect();
    startProcessPolling();
    startMetricsHistory();
    fetchHealth().catch(console.error);

    healthCheckInterval = setInterval(() => {
      if ($connected) {
        fetchHealth().catch(console.error);
      }
    }, 30000);
  });

  onDestroy(() => {
    stopMetricsHistory();
    stopProcessPolling();
    disconnect();
    clearInterval(healthCheckInterval);
  });
</script>

<div class="dashboard">
  <Header />

  <div class="metric-bar">
    <MetricPill label="CPU" value={cpuPercent} unit="%" color="cpu" />
    <MetricPill label="MEM" value={memPercent} unit="%" color="memory" />
    <MetricPill label="LOAD" value={loadAvg1m} color="load" />
    <MetricPill label="NET" value={netRx} icon="↓" extra={netTx} extraIcon="↑" color="net" />
    <MetricPill label="DISK" value={diskPercent} unit="%" color="disk" />
    <MetricPill label="TEMP" value={avgTemp} unit="°C" color="default" />
    <MetricPill label="GPU" value={gpuUtil} unit="%" color="default" />
    <MetricPill label="PORTS" value={portCount} color="default" />
    <MetricPill label="●" value={containerCount} color="default" />
  </div>

  <div class="main-area">
    <Sidebar />

    <main class="workspace">
      {#if $activeView === 'overview'}
        <OverviewView />
      {:else if $activeView === 'processes'}
        <ProcessesView />
      {:else if $activeView === 'network'}
        <NetworkView />
      {:else if $activeView === 'containers'}
        <ContainersView />
      {:else if $activeView === 'alerts'}
        <AlertsView />
      {/if}
    </main>
  </div>
</div>

<style>
  .dashboard {
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .metric-bar {
    display: flex;
    justify-content: center;
    gap: 0.125rem;
    padding: 0.25rem 0.75rem;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
    overflow-x: auto;
  }

  .metric-bar::-webkit-scrollbar {
    display: none;
  }

  .main-area {
    flex: 1;
    display: flex;
    overflow: hidden;
    min-height: 0;
  }

  .workspace {
    flex: 1;
    overflow: hidden;
    min-width: 0;
    background: var(--bg-primary);
  }

  /* Mobile: sidebar goes to bottom */
  @media (max-width: 768px) {
    .main-area {
      flex-direction: column;
    }

    .metric-bar {
      padding: 4px 8px;
    }
  }
</style>

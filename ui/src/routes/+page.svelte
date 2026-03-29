<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { connect, disconnect, fetchHealth, connected, metrics, alerts } from '$lib/stores/metrics';
  import Header from '$lib/components/Header.svelte';
  import MetricPill from '$lib/components/MetricPill.svelte';
  import AlertsCompact from '$lib/components/AlertsCompact.svelte';
  import CpuCoresCompact from '$lib/components/CpuCoresCompact.svelte';
  import MemoryCompact from '$lib/components/MemoryCompact.svelte';
  import LoadCompact from '$lib/components/LoadCompact.svelte';
  import DiskCompact from '$lib/components/DiskCompact.svelte';
  import NetworkCompact from '$lib/components/NetworkCompact.svelte';
  import ContainersCompact from '$lib/components/ContainersCompact.svelte';
  import StaleCompact from '$lib/components/StaleCompact.svelte';
  import ProcessTable from '$lib/components/ProcessTable.svelte';

  let healthCheckInterval: ReturnType<typeof setInterval>;

  // Derived values for metric pills
  const cpuPercent = $derived($metrics?.cpu.overall_percent ?? 0);
  const memPercent = $derived($metrics?.memory.used_percent ?? 0);
  const loadAvg1m = $derived($metrics?.load_avg[0] ?? 0);
  const netRx = $derived($metrics?.network.total_received_bytes ?? 0);
  const netTx = $derived($metrics?.network.total_transmitted_bytes ?? 0);
  const diskPercent = $derived($metrics?.disk.disks[0]?.used_percent ?? 0);
  const containerCount = $derived($metrics?.system?.container_count ?? 0);

  onMount(() => {
    connect();
    fetchHealth().catch(console.error);

    healthCheckInterval = setInterval(() => {
      if ($connected) {
        fetchHealth().catch(console.error);
      }
    }, 30000);
  });

  onDestroy(() => {
    disconnect();
    clearInterval(healthCheckInterval);
  });
</script>

<div class="dashboard">
  <Header />

  <!-- Top Metric Bar -->
  <div class="metric-bar">
    <MetricPill label="CPU" value={cpuPercent} unit="%" color="cpu" />
    <MetricPill label="MEM" value={memPercent} unit="%" color="memory" />
    <MetricPill label="LOAD" value={loadAvg1m} color="load" />
    <MetricPill label="NET" value={netRx} icon="↓" extra={netTx} extraIcon="↑" color="net" />
    <MetricPill label="DISK" value={diskPercent} unit="%" color="disk" />
    <MetricPill label="●" value={containerCount} color="default" />
  </div>

  <!-- Compact Alert Bar -->
  <AlertsCompact />

  <!-- Two-Column Main Grid -->
  <div class="main-grid">
    <!-- LEFT COLUMN: CPU + Processes -->
    <div class="left-column">
      <CpuCoresCompact />
      <ProcessTable />
    </div>

    <!-- RIGHT COLUMN: Everything Else -->
    <div class="right-column">
      <MemoryCompact />
      <LoadCompact />
      <DiskCompact />
      <NetworkCompact />
      <ContainersCompact />
      <StaleCompact />
    </div>
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
    gap: 2px;
    padding: 4px 12px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
    overflow-x: auto;
  }

  .metric-bar::-webkit-scrollbar {
    display: none;
  }

  .main-grid {
    flex: 1;
    display: grid;
    grid-template-columns: 58% 42%;
    overflow: hidden;
    min-height: 0;
  }

  .left-column {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border-right: 1px solid var(--border-color);
  }

  .right-column {
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    padding: 6px;
    gap: 6px;
  }

  /* Responsive adjustments */
  @media (max-width: 1200px) {
    .main-grid {
      grid-template-columns: 55% 45%;
    }
  }

  @media (max-width: 1024px) {
    .main-grid {
      grid-template-columns: 1fr;
      grid-template-rows: auto 1fr;
    }

    .left-column {
      border-right: none;
      border-bottom: 1px solid var(--border-color);
      max-height: 50vh;
    }

    .right-column {
      flex-direction: row;
      flex-wrap: wrap;
      overflow-y: auto;
    }

    .right-column :global(> *) {
      flex: 1;
      min-width: 200px;
    }
  }

  @media (max-width: 640px) {
    .metric-bar {
      padding: 4px 8px;
    }

    .main-grid {
      grid-template-columns: 1fr;
      grid-template-rows: auto 1fr;
    }

    .left-column {
      max-height: 60vh;
    }

    .right-column {
      flex-direction: column;
    }

    .right-column :global(> *) {
      min-width: unset;
    }
  }
</style>

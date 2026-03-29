<script lang="ts">
  import CpuCoresCompact from '$lib/components/CpuCoresCompact.svelte';
  import MemoryCompact from '$lib/components/MemoryCompact.svelte';
  import LoadCompact from '$lib/components/LoadCompact.svelte';
  import DiskCompact from '$lib/components/DiskCompact.svelte';
  import NetworkCompact from '$lib/components/NetworkCompact.svelte';
  import PortsCompact from '$lib/components/PortsCompact.svelte';
  import ContainersCompact from '$lib/components/ContainersCompact.svelte';
  import StaleCompact from '$lib/components/StaleCompact.svelte';
  import AlertsCompact from '$lib/components/AlertsCompact.svelte';
  import { setView } from '$lib/stores/navigation';
</script>

<div class="overview">
  <AlertsCompact />

  <div class="grid">
    <button class="card wide" onclick={() => setView('processes')}>
      <CpuCoresCompact />
    </button>

    <div class="card">
      <MemoryCompact />
    </div>

    <div class="card">
      <LoadCompact />
    </div>

    <div class="card">
      <DiskCompact />
    </div>

    <button class="card" onclick={() => setView('network')}>
      <NetworkCompact />
    </button>

    <button class="card" onclick={() => setView('network')}>
      <PortsCompact />
    </button>

    <button class="card" onclick={() => setView('containers')}>
      <ContainersCompact />
    </button>

    <div class="card">
      <StaleCompact />
    </div>
  </div>
</div>

<style>
  .overview {
    height: 100%;
    overflow-y: auto;
    padding: 8px;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 8px;
    margin-top: 8px;
  }

  .card {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    overflow: hidden;
    text-align: left;
    color: inherit;
    cursor: default;
    padding: 0;
  }

  button.card {
    cursor: pointer;
    transition: border-color var(--transition-fast);
  }

  button.card:hover {
    border-color: var(--accent);
  }

  .card.wide {
    grid-column: span 2;
  }

  .card :global(> *) {
    border: none;
    border-radius: 0;
  }

  @media (max-width: 768px) {
    .grid {
      grid-template-columns: 1fr;
    }

    .card.wide {
      grid-column: span 1;
    }
  }
</style>

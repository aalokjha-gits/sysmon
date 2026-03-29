<script lang="ts">
  import { onMount } from 'svelte';
  import { fetchContainers, restartContainer } from '$lib/stores/metrics';
  import { formatBytes } from '$lib/utils/format';
  import ConfirmModal from './ConfirmModal.svelte';
  import type { Container, ContainersResponse } from '$lib/types';

  let containersData = $state<ContainersResponse | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let selectedContainer: Container | null = $state(null);
  let showRestartModal = $state(false);
  let restarting = $state(false);
  let actionResult = $state<{ success: boolean; message: string } | null>(null);

  async function loadContainers() {
    loading = true;
    error = null;

    try {
      containersData = await fetchContainers();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load containers';
    } finally {
      loading = false;
    }
  }

  function handleRestart(container: Container) {
    selectedContainer = container;
    showRestartModal = true;
  }

  async function confirmRestart() {
    if (!selectedContainer) return;

    restarting = true;

    try {
      await restartContainer(selectedContainer.id);
      actionResult = {
        success: true,
        message: `Container "${selectedContainer.name}" restarted successfully`
      };
      showRestartModal = false;
      setTimeout(loadContainers, 2000);
    } catch (err) {
      actionResult = {
        success: false,
        message: err instanceof Error ? err.message : 'Failed to restart container'
      };
    } finally {
      restarting = false;
      selectedContainer = null;
    }
  }

  function cancelRestart() {
    showRestartModal = false;
    selectedContainer = null;
  }

  function getStatusColor(status: string): string {
    const s = status.toLowerCase();
    if (s === 'running') return 'var(--success)';
    if (s === 'paused') return 'var(--warning)';
    if (s === 'exited' || s === 'stopped' || s === 'dead') return 'var(--danger)';
    if (s === 'restarting' || s === 'created') return 'var(--info)';
    return 'var(--text-muted)';
  }

  function getStatusIcon(status: string): string {
    const s = status.toLowerCase();
    if (s === 'running') return '●';
    if (s === 'paused') return '⏸';
    if (s === 'exited' || s === 'stopped' || s === 'dead') return '○';
    if (s === 'restarting') return '↻';
    return '◌';
  }

  function getActionIcon(container: Container): string {
    const status = container.status.toLowerCase();
    if (status === 'running') return '↻';
    return '▶';
  }

  onMount(() => {
    loadContainers();
    const interval = setInterval(loadContainers, 30000);
    return () => clearInterval(interval);
  });

  const containerCount = $derived(containersData?.containers.length ?? 0);
</script>

<div class="containers-compact compact-panel">
  <div class="panel-header">
    <span class="compact-panel-title">Containers ({containerCount})</span>
    <button class="refresh-btn" onclick={loadContainers} disabled={loading} aria-label="Refresh">
      <svg class:spinning={loading} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="23 4 23 10 17 10" />
        <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
      </svg>
    </button>
  </div>

  {#if loading && !containersData}
    <div class="containers-list">
      <div class="container-row skeleton">
        <span class="status-icon">●</span>
        <span class="container-name">Loading...</span>
        <span class="container-state">--</span>
        <span class="container-cpu mono">--</span>
        <span class="container-mem mono">--</span>
      </div>
    </div>
  {:else if containersData?.runtime === null}
    <div class="empty-state">
      No container runtime
    </div>
  {:else if containersData && containersData.containers.length === 0}
    <div class="empty-state">
      No containers running
    </div>
  {:else if containersData}
    <div class="containers-list">
      {#each containersData.containers as container (container.id)}
        <div class="container-row">
          <span class="status-icon" style:color={getStatusColor(container.status)}>
            {getStatusIcon(container.status)}
          </span>
          <span class="container-name" title={container.name}>{container.name}</span>
          <span class="container-state">{container.status}</span>
          <span class="container-cpu mono">{container.cpu_percent.toFixed(1)}%</span>
          <span class="container-mem mono">{formatBytes(container.memory_bytes)}</span>
          <button
            class="action-btn"
            onclick={() => handleRestart(container)}
            disabled={restarting}
            title="{container.status.toLowerCase() === 'running' ? 'Restart' : 'Start'} container"
          >
            {getActionIcon(container)}
          </button>
        </div>
      {/each}
    </div>
  {/if}
</div>

<ConfirmModal
  isOpen={showRestartModal}
  title="Restart Container"
  message={selectedContainer
    ? `Are you sure you want to restart "${selectedContainer.name}"? This will briefly interrupt the service.`
    : ''}
  confirmText={restarting ? 'Restarting...' : 'Restart'}
  cancelText="Cancel"
  variant="warning"
  onConfirm={confirmRestart}
  onCancel={cancelRestart}
/>

{#if actionResult}
  <div class="result-toast" class:error={!actionResult.success}>
    <span>{actionResult.message}</span>
    <button onclick={() => actionResult = null}>×</button>
  </div>
{/if}

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

  .refresh-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 2px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color var(--transition-fast);
  }

  .refresh-btn:hover {
    color: var(--text-primary);
  }

  .refresh-btn svg {
    width: 12px;
    height: 12px;
  }

  .refresh-btn svg.spinning {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .containers-list {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .container-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: var(--font-xs);
    padding: 3px 0;
  }

  .container-row.skeleton {
    opacity: 0.5;
  }

  .status-icon {
    font-size: 8px;
    width: 12px;
    text-align: center;
  }

  .container-name {
    flex: 1;
    color: var(--text-primary);
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 60px;
  }

  .container-state {
    color: var(--text-secondary);
    min-width: 50px;
  }

  .container-cpu {
    min-width: 35px;
    text-align: right;
    color: var(--text-secondary);
  }

  .container-mem {
    min-width: 50px;
    text-align: right;
    color: var(--text-secondary);
  }

  .action-btn {
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--text-muted);
    padding: 1px 5px;
    border-radius: 3px;
    font-size: 10px;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .action-btn:hover {
    background: var(--bg-elevated);
    color: var(--text-primary);
    border-color: var(--accent);
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .empty-state {
    font-size: var(--font-xs);
    color: var(--text-muted);
    padding: 8px 0;
    text-align: center;
  }

  .mono {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }

  .result-toast {
    position: fixed;
    bottom: var(--space-4);
    right: var(--space-4);
    background: var(--success);
    color: white;
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    gap: var(--space-3);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    z-index: 1000;
    animation: fadeIn var(--transition-normal);
  }

  .result-toast.error {
    background: var(--danger);
  }

  .result-toast button {
    background: none;
    border: none;
    color: white;
    font-size: 1.25rem;
    cursor: pointer;
    line-height: 1;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>

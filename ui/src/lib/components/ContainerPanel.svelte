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
      // Reload containers after restart
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

  function getActionButton(container: Container): { label: string; action: () => void } {
    const status = container.status.toLowerCase();

    if (status === 'running') {
      return {
        label: 'Restart',
        action: () => handleRestart(container)
      };
    }

    return {
      label: 'Start',
      action: () => handleRestart(container)
    };
  }

  onMount(() => {
    loadContainers();
    // Refresh every 30 seconds
    const interval = setInterval(loadContainers, 30000);
    return () => clearInterval(interval);
  });
</script>

<div class="container-panel">
  <div class="panel-header">
    <h3 class="panel-title">
      <svg class="title-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
        <polyline points="3.27 6.96 12 12.01 20.73 6.96" />
        <line x1="12" y1="22.08" x2="12" y2="12" />
      </svg>
      Containers
      {#if containersData?.containers.length}
        <span class="container-count">({containersData.containers.length})</span>
      {/if}
    </h3>
    <button class="refresh-btn" onclick={loadContainers} disabled={loading} aria-label="Refresh containers">
      <svg class:spinning={loading} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="23 4 23 10 17 10" />
        <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
      </svg>
    </button>
  </div>

  {#if loading && !containersData}
    <div class="loading-state">
      <div class="spinner"></div>
      <span>Loading containers...</span>
    </div>
  {:else if error}
    <div class="error-state">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10" />
        <line x1="12" y1="8" x2="12" y2="12" />
        <line x1="12" y1="16" x2="12.01" y2="16" />
      </svg>
      <span>{error}</span>
      <button onclick={loadContainers}>Retry</button>
    </div>
  {:else if containersData?.runtime === null}
    <div class="empty-state">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
        <polyline points="3.27 6.96 12 12.01 20.73 6.96" />
        <line x1="12" y1="22.08" x2="12" y2="12" />
      </svg>
      <p>No container runtime detected</p>
      <span class="empty-hint">Docker or Podman not available on this system</span>
    </div>
  {:else if containersData && containersData.containers.length === 0}
    <div class="empty-state">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
        <polyline points="3.27 6.96 12 12.01 20.73 6.96" />
        <line x1="12" y1="22.08" x2="12" y2="12" />
      </svg>
      <p>No containers running</p>
      <span class="empty-hint">Runtime: {containersData.runtime}</span>
    </div>
  {:else if containersData}
    <div class="containers-grid">
      {#each containersData.containers as container (container.id)}
        <div class="container-card">
          <div class="card-header-row">
            <div class="status-indicator" style:background-color={getStatusColor(container.status)}></div>
            <span class="container-name" title={container.name}>{container.name}</span>
          </div>

          <div class="card-image">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
              <polyline points="3.27 6.96 12 12.01 20.73 6.96" />
              <line x1="12" y1="22.08" x2="12" y2="12" />
            </svg>
            <span class="image-name" title={container.image}>{container.image.split(':')[0]}</span>
          </div>

          <div class="card-stats">
            <div class="stat">
              <span class="stat-label">Status</span>
              <span class="stat-value" style:color={getStatusColor(container.status)}>
                {container.status}
              </span>
            </div>
            <div class="stat">
              <span class="stat-label">CPU</span>
              <span class="stat-value">{container.cpu_percent.toFixed(1)}%</span>
            </div>
            <div class="stat">
              <span class="stat-label">Memory</span>
              <span class="stat-value">{formatBytes(container.memory_bytes)}</span>
            </div>
            {#if container.ports.length > 0}
              <div class="stat full-width">
                <span class="stat-label">Ports</span>
                <span class="stat-value ports">{container.ports.slice(0, 2).join(', ')}{container.ports.length > 2 ? '...' : ''}</span>
              </div>
            {/if}
          </div>

          <button
            class="action-btn"
            class:restart={container.status.toLowerCase() === 'running'}
            onclick={getActionButton(container).action}
            disabled={restarting}
          >
            {getActionButton(container).label}
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
  <div class="result-toast animate-fade-in" class:error={!actionResult.success}>
    <span>{actionResult.message}</span>
    <button onclick={() => actionResult = null}>×</button>
  </div>
{/if}

<style>
  .container-panel {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
    margin: var(--space-4) var(--space-6) 0;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-4);
  }

  .panel-title {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .title-icon {
    width: 18px;
    height: 18px;
    color: var(--accent);
  }

  .container-count {
    color: var(--text-muted);
    font-weight: 400;
  }

  .refresh-btn {
    background: var(--bg-elevated);
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    padding: var(--space-2);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .refresh-btn:hover {
    background: var(--bg-surface-hover);
    color: var(--text-primary);
  }

  .refresh-btn svg {
    width: 16px;
    height: 16px;
  }

  .refresh-btn svg.spinning {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .loading-state,
  .error-state,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-8);
    color: var(--text-secondary);
    text-align: center;
  }

  .error-state svg,
  .empty-state svg {
    width: 48px;
    height: 48px;
    margin-bottom: var(--space-3);
    color: var(--text-muted);
  }

  .error-state svg {
    color: var(--danger);
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 2px solid var(--border-color);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: var(--space-3);
  }

  .error-state button,
  .empty-state p {
    margin-top: var(--space-3);
  }

  .error-state button {
    background: var(--danger);
    color: white;
    border: none;
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-md);
    font-size: 0.875rem;
    cursor: pointer;
  }

  .empty-hint {
    font-size: 0.8125rem;
    color: var(--text-muted);
    margin-top: var(--space-1);
  }

  .containers-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: var(--space-4);
  }

  .container-card {
    background: var(--bg-elevated);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
    transition: all var(--transition-fast);
  }

  .container-card:hover {
    border-color: var(--border-color);
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }

  .card-header-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-3);
  }

  .status-indicator {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
    box-shadow: 0 0 6px currentColor;
  }

  .container-name {
    font-weight: 600;
    font-size: 0.9375rem;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .card-image {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: var(--space-3);
    margin-bottom: var(--space-3);
    background: var(--bg-surface);
    border-radius: var(--radius-md);
  }

  .card-image svg {
    width: 40px;
    height: 40px;
    color: var(--text-muted);
    margin-bottom: var(--space-2);
  }

  .image-name {
    font-size: 0.75rem;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
  }

  .card-stats {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-2);
    margin-bottom: var(--space-3);
  }

  .stat {
    display: flex;
    flex-direction: column;
  }

  .stat.full-width {
    grid-column: span 2;
  }

  .stat-label {
    font-size: 0.6875rem;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.025em;
  }

  .stat-value {
    font-size: 0.8125rem;
    color: var(--text-primary);
    font-weight: 500;
  }

  .stat-value.ports {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .action-btn {
    width: 100%;
    padding: var(--space-2);
    border-radius: var(--radius-md);
    font-size: 0.8125rem;
    font-weight: 500;
    cursor: pointer;
    transition: all var(--transition-fast);
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
  }

  .action-btn:hover {
    background: var(--bg-surface-hover);
    color: var(--text-primary);
  }

  .action-btn.restart {
    background: rgba(59, 130, 246, 0.1);
    border-color: var(--accent);
    color: var(--accent);
  }

  .action-btn.restart:hover {
    background: rgba(59, 130, 246, 0.2);
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
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

  @media (max-width: 640px) {
    .container-panel {
      margin: var(--space-3) var(--space-4) 0;
      padding: var(--space-3);
    }

    .containers-grid {
      grid-template-columns: 1fr;
    }
  }
</style>

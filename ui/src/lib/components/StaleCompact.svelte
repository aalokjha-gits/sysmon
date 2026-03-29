<script lang="ts">
  import { onMount } from 'svelte';
  import { fetchStaleProcesses, cleanup } from '$lib/stores/metrics';
  import { formatBytes } from '$lib/utils/format';
  import ConfirmModal from './ConfirmModal.svelte';
  import type { StaleResponse } from '$lib/types';

  let staleData = $state<StaleResponse | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let expanded = $state(false);
  let showCleanupModal = $state(false);
  let cleaning = $state(false);
  let cleanupResult = $state<{ success: boolean; message: string } | null>(null);
  let showDryRun = $state(false);
  let dryRunResult = $state<{
    freed_bytes: number;
    processes: { pid: number; name: string; process_type: string }[];
    total_killed: number;
  } | null>(null);

  async function loadStaleProcesses() {
    loading = true;
    error = null;

    try {
      staleData = await fetchStaleProcesses();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load stale processes';
    } finally {
      loading = false;
    }
  }

  async function runDryRun() {
    cleaning = true;
    error = null;

    try {
      const result = await cleanup(true);
      dryRunResult = {
        freed_bytes: result.freed_bytes,
        processes: result.processes,
        total_killed: result.total_killed
      };
      showDryRun = true;
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to run cleanup preview';
    } finally {
      cleaning = false;
    }
  }

  async function confirmCleanup() {
    cleaning = true;
    error = null;

    try {
      const result = await cleanup(false);
      cleanupResult = {
        success: true,
        message: `Killed ${result.total_killed} processes and freed ${formatBytes(result.freed_bytes)}`
      };
      showCleanupModal = false;
      showDryRun = false;
      dryRunResult = null;
      await loadStaleProcesses();
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to cleanup processes';
      cleanupResult = { success: false, message: error };
    } finally {
      cleaning = false;
    }
  }

  function cancelCleanup() {
    showCleanupModal = false;
  }

  function closeDryRun() {
    showDryRun = false;
    dryRunResult = null;
  }

  function proceedToCleanup() {
    showDryRun = false;
    showCleanupModal = true;
  }

  function toggleExpanded() {
    expanded = !expanded;
  }

  onMount(() => {
    loadStaleProcesses();
    const interval = setInterval(loadStaleProcesses, 30000);
    return () => clearInterval(interval);
  });

  const totalStaleCount = $derived((staleData?.stale_count ?? 0) + (staleData?.zombie_count ?? 0));
  const hasStale = $derived(totalStaleCount > 0);
  const severityColor = $derived(staleData?.zombie_count ? 'var(--danger)' : hasStale ? 'var(--warning)' : 'var(--text-muted)');
</script>

<div class="stale-compact compact-panel">
  <div class="stale-bar">
    <button class="stale-toggle" onclick={toggleExpanded} aria-expanded={expanded}>
      <span class="stale-icon" style:color={severityColor}>
        {#if staleData?.zombie_count}
          ☠
        {:else if hasStale}
          ⚠
        {:else}
          ✓
        {/if}
      </span>
      <span class="stale-text">
        {#if staleData}
          {#if hasStale}
            {totalStaleCount} stale ({formatBytes(staleData.total_memory_waste_bytes)} waste)
          {:else}
            No stale processes
          {/if}
        {:else if loading}
          Loading...
        {:else}
          No stale processes
        {/if}
      </span>
      {#if hasStale}
        <svg
          class="expand-icon"
          class:expanded={expanded}
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
      {/if}
    </button>

    {#if hasStale}
      <div class="stale-actions">
        <button class="action-btn preview" onclick={runDryRun} disabled={cleaning}>
          Preview
        </button>
        <button
          class="action-btn cleanup"
          class:has-zombies={staleData?.zombie_count}
          onclick={() => showCleanupModal = true}
          disabled={cleaning}
        >
          {cleaning ? '...' : 'Clean Up'}
        </button>
      </div>
    {/if}
  </div>

  {#if expanded && staleData && staleData.processes.length > 0}
    <div class="stale-list">
      {#each staleData.processes.slice(0, 10) as process}
        <div class="stale-item">
          <span class="mono pid">{process.pid}</span>
          <span class="name">{process.name}</span>
          <span class="reason {process.stale_reason}">{process.stale_reason}</span>
          <span class="mono mem">{formatBytes(process.memory_bytes)}</span>
        </div>
      {/each}
      {#if staleData.processes.length > 10}
        <div class="more-text">
          +{staleData.processes.length - 10} more
        </div>
      {/if}
    </div>
  {/if}

  {#if error}
    <div class="stale-error">
      <span>{error}</span>
      <button onclick={() => loadStaleProcesses()}>Retry</button>
    </div>
  {/if}
</div>

<!-- Dry Run Modal -->
<ConfirmModal
  isOpen={showDryRun}
  title="Cleanup Preview"
  message={dryRunResult
    ? `This action will terminate ${dryRunResult.total_killed} process${dryRunResult.total_killed !== 1 ? 'es' : ''} and free approximately ${formatBytes(dryRunResult.freed_bytes)} of memory.`
    : ''}
  confirmText="Proceed to Cleanup"
  cancelText="Cancel"
  variant="warning"
  onConfirm={proceedToCleanup}
  onCancel={closeDryRun}
/>

<!-- Cleanup Confirmation Modal -->
<ConfirmModal
  isOpen={showCleanupModal}
  title="Clean Up Stale Processes"
  message="Are you sure you want to terminate all stale and zombie processes? This action cannot be undone and may affect running applications."
  confirmText={cleaning ? 'Cleaning...' : 'Clean Up All'}
  cancelText="Cancel"
  variant="danger"
  onConfirm={confirmCleanup}
  onCancel={cancelCleanup}
/>

{#if cleanupResult}
  <div class="result-toast" class:error={!cleanupResult.success}>
    <span>{cleanupResult.message}</span>
    <button onclick={() => cleanupResult = null}>×</button>
  </div>
{/if}

<style>
  .compact-panel {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    padding: 6px 10px;
    overflow: hidden;
  }

  .stale-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .stale-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
    background: transparent;
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    font-size: var(--font-sm);
    padding: 0;
    text-align: left;
  }

  .stale-toggle:hover {
    opacity: 0.9;
  }

  .stale-icon {
    font-size: 12px;
  }

  .stale-text {
    flex: 1;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .expand-icon {
    width: 14px;
    height: 14px;
    color: var(--text-muted);
    transition: transform var(--transition-fast);
  }

  .expand-icon.expanded {
    transform: rotate(180deg);
  }

  .stale-actions {
    display: flex;
    gap: 4px;
  }

  .action-btn {
    padding: 2px 8px;
    border-radius: 3px;
    font-size: var(--font-xs);
    font-weight: 500;
    cursor: pointer;
    transition: all var(--transition-fast);
    border: 1px solid;
  }

  .action-btn.preview {
    background: transparent;
    border-color: var(--border-color);
    color: var(--text-secondary);
  }

  .action-btn.preview:hover {
    background: var(--bg-elevated);
    color: var(--text-primary);
  }

  .action-btn.cleanup {
    background: var(--warning);
    border-color: var(--warning);
    color: white;
  }

  .action-btn.cleanup.has-zombies {
    background: var(--danger);
    border-color: var(--danger);
  }

  .action-btn.cleanup:hover {
    filter: brightness(1.1);
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .stale-list {
    margin-top: 6px;
    padding-top: 6px;
    border-top: 1px solid var(--border-color);
  }

  .stale-item {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: var(--font-xs);
    padding: 2px 0;
    overflow: hidden;
  }

  .pid {
    min-width: 45px;
    color: var(--text-muted);
  }

  .name {
    flex: 1;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .reason {
    min-width: 50px;
    text-transform: uppercase;
    font-size: 10px;
    padding: 1px 4px;
    border-radius: 2px;
    background: rgba(245, 158, 11, 0.2);
    color: var(--warning);
  }

  .reason.zombie {
    background: rgba(239, 68, 68, 0.2);
    color: var(--danger);
  }

  .mem {
    min-width: 50px;
    text-align: right;
    color: var(--text-secondary);
  }

  .more-text {
    font-size: var(--font-xs);
    color: var(--text-muted);
    padding: 4px 0;
    text-align: center;
  }

  .stale-error {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 6px;
    padding: 6px 8px;
    background: rgba(239, 68, 68, 0.1);
    color: var(--danger);
    font-size: var(--font-xs);
    border-radius: 3px;
  }

  .stale-error button {
    background: var(--danger);
    color: white;
    border: none;
    padding: 2px 8px;
    border-radius: 3px;
    font-size: var(--font-xs);
    cursor: pointer;
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

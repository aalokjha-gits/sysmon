<script lang="ts">
  import { onMount } from 'svelte';
  import { fetchStaleProcesses, cleanup } from '$lib/stores/metrics';
  import { formatBytes, formatDuration } from '$lib/utils/format';
  import ConfirmModal from './ConfirmModal.svelte';
  import type { StaleResponse, StaleProcess } from '$lib/types';

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
      // Reload stale data
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
    // Refresh every 30 seconds
    const interval = setInterval(loadStaleProcesses, 30000);
    return () => clearInterval(interval);
  });

  const totalStaleCount = $derived((staleData?.stale_count ?? 0) + (staleData?.zombie_count ?? 0));
  const hasStale = $derived(totalStaleCount > 0);
  const severityColor = $derived(staleData?.zombie_count ? 'var(--danger)' : hasStale ? 'var(--warning)' : 'var(--success)');
  const severityBg = $derived(staleData?.zombie_count ? 'rgba(239, 68, 68, 0.1)' : hasStale ? 'rgba(245, 158, 11, 0.1)' : 'rgba(34, 197, 94, 0.1)');
</script>

<div class="stale-panel" class:has-content={hasStale} style:background-color={severityBg} style:border-color={severityColor}>
  <div class="panel-header">
    <button class="panel-toggle" onclick={toggleExpanded} aria-expanded={expanded}>
      <div class="panel-left">
        <svg class="panel-icon" viewBox="0 0 24 24" fill="none" stroke={severityColor} stroke-width="2">
          <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
          <line x1="12" y1="9" x2="12" y2="13" />
          <line x1="12" y1="17" x2="12.01" y2="17" />
        </svg>
        <div class="panel-info">
          <span class="panel-title" style:color={severityColor}>
            {#if staleData}
              {staleData.zombie_count > 0 ? '⚠ ' : ''}
              {totalStaleCount} stale process{totalStaleCount !== 1 ? 'es' : ''}
              {staleData.total_memory_waste_bytes > 0 ? `wasting ${formatBytes(staleData.total_memory_waste_bytes)}` : ''}
            {:else if loading}
              Loading...
            {:else}
              No stale processes
            {/if}
          </span>
          {#if staleData && staleData.zombie_count > 0}
            <span class="panel-subtitle">{staleData.zombie_count} zombie process{staleData.zombie_count !== 1 ? 'es' : ''} detected</span>
          {/if}
        </div>
      </div>

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
    </button>

    {#if hasStale}
      <div class="panel-actions">
        <button
          class="preview-btn"
          onclick={runDryRun}
          disabled={cleaning}
        >
          Preview
        </button>
        <button
          class="cleanup-btn"
          class:has-zombies={staleData?.zombie_count}
          onclick={() => showCleanupModal = true}
          disabled={cleaning}
        >
          {cleaning ? 'Cleaning...' : 'Clean Up'}
        </button>
      </div>
    {/if}
  </div>

  {#if expanded && staleData && staleData.processes.length > 0}
    <div class="panel-content animate-slide-down">
      <table class="stale-table">
        <thead>
          <tr>
            <th>PID</th>
            <th>Name</th>
            <th>Reason</th>
            <th>Age</th>
            <th>Memory</th>
            <th>Dupes</th>
          </tr>
        </thead>
        <tbody>
          {#each staleData.processes.slice(0, 20) as process}
            <tr>
              <td class="pid-cell">{process.pid}</td>
              <td class="name-cell">{process.name}</td>
              <td class="reason-cell">
                <span class="reason-badge" class:zombie={process.stale_reason === 'zombie'}>
                  {process.stale_reason}
                </span>
              </td>
              <td class="numeric-cell">{formatDuration(process.runtime_hours * 3600)}</td>
              <td class="numeric-cell">{formatBytes(process.memory_bytes)}</td>
              <td class="numeric-cell">
                {#if process.duplicate_count > 1}
                  <span class="dupes-badge">{process.duplicate_count}</span>
                {:else}
                  -
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
      {#if staleData.processes.length > 20}
        <div class="more-processes">
          +{staleData.processes.length - 20} more processes
        </div>
      {/if}
    </div>
  {/if}

  {#if error}
    <div class="panel-error">
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
  <div class="result-toast animate-fade-in" class:error={!cleanupResult.success}>
    <span>{cleanupResult.message}</span>
    <button onclick={() => cleanupResult = null}>×</button>
  </div>
{/if}

<style>
  .stale-panel {
    border: 1px solid;
    border-left: 4px solid;
    border-radius: var(--radius-lg);
    margin: var(--space-4) var(--space-6) 0;
    overflow: hidden;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: var(--space-3) var(--space-4);
    background: transparent;
    color: var(--text-primary);
  }

  .panel-toggle {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex: 1;
    background: transparent;
    border: none;
    cursor: pointer;
    color: inherit;
    padding: 0;
    margin-right: var(--space-3);
    transition: opacity var(--transition-fast);
  }

  .panel-toggle:hover {
    opacity: 0.8;
  }

  .panel-left {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .panel-icon {
    width: 24px;
    height: 24px;
    flex-shrink: 0;
  }

  .panel-info {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
  }

  .panel-title {
    font-weight: 600;
    font-size: 0.9375rem;
  }

  .panel-subtitle {
    font-size: 0.8125rem;
    color: var(--text-secondary);
  }

  .panel-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .preview-btn,
  .cleanup-btn {
    padding: var(--space-1) var(--space-3);
    border-radius: var(--radius-md);
    font-size: 0.8125rem;
    font-weight: 500;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .preview-btn {
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
  }

  .preview-btn:hover {
    background: var(--bg-surface-hover);
    color: var(--text-primary);
  }

  .cleanup-btn {
    background: var(--warning);
    border: 1px solid var(--warning);
    color: white;
  }

  .cleanup-btn.has-zombies {
    background: var(--danger);
    border-color: var(--danger);
  }

  .cleanup-btn:hover {
    filter: brightness(1.1);
  }

  .cleanup-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .expand-icon {
    width: 20px;
    height: 20px;
    color: var(--text-secondary);
    transition: transform var(--transition-fast);
  }

  .expand-icon.expanded {
    transform: rotate(180deg);
  }

  .panel-content {
    padding: 0 var(--space-4) var(--space-4);
  }

  .stale-table {
    width: 100%;
    font-size: 0.8125rem;
    border-collapse: collapse;
  }

  th, td {
    padding: var(--space-2) var(--space-3);
    text-align: left;
    border-bottom: 1px solid var(--border-color);
  }

  th {
    color: var(--text-secondary);
    font-weight: 500;
  }

  td {
    color: var(--text-primary);
  }

  .pid-cell {
    font-family: 'SF Mono', Monaco, monospace;
    color: var(--text-muted);
  }

  .numeric-cell {
    text-align: right;
    font-family: 'SF Mono', Monaco, monospace;
  }

  .reason-badge {
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
    font-size: 0.75rem;
    background: rgba(245, 158, 11, 0.2);
    color: var(--warning);
  }

  .reason-badge.zombie {
    background: rgba(239, 68, 68, 0.2);
    color: var(--danger);
  }

  .dupes-badge {
    background: var(--accent);
    color: white;
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
    font-size: 0.75rem;
    font-weight: 600;
  }

  .more-processes {
    text-align: center;
    padding: var(--space-3);
    color: var(--text-muted);
    font-size: 0.8125rem;
  }

  .panel-error {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3) var(--space-4);
    background: rgba(239, 68, 68, 0.1);
    color: var(--danger);
    font-size: 0.875rem;
  }

  .panel-error button {
    background: var(--danger);
    color: white;
    border: none;
    padding: var(--space-1) var(--space-3);
    border-radius: var(--radius-md);
    font-size: 0.8125rem;
    cursor: pointer;
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
    .stale-panel {
      margin: var(--space-3) var(--space-4) 0;
    }

    .panel-header {
      flex-wrap: wrap;
      gap: var(--space-2);
    }

    .panel-actions {
      width: 100%;
      justify-content: flex-end;
    }

    .stale-table {
      font-size: 0.75rem;
    }

    th, td {
      padding: var(--space-1) var(--space-2);
    }
  }
</style>

<script lang="ts">
  import { processes } from '$lib/stores/metrics';
  import { killProcess } from '$lib/stores/metrics';
  import { formatBytes, formatPercent, formatDuration, truncate } from '$lib/utils/format';
  import ConfirmModal from './ConfirmModal.svelte';
  import type { ProcessInfo } from '$lib/types';

  type SortKey = 'pid' | 'name' | 'cpu_percent' | 'memory_bytes' | 'status' | 'runtime_seconds';
  type SortDirection = 'asc' | 'desc';

  let sortKey = $state<SortKey>('cpu_percent');
  let sortDirection = $state<SortDirection>('desc');
  let searchQuery = $state('');
  let showAll = $state(false);
  let selectedProcess: ProcessInfo | null = $state(null);
  let showKillModal = $state(false);
  let killing = $state(false);
  let killError = $state<string | null>(null);

  const DISPLAY_LIMIT = 50;

  function toggleSort(key: SortKey) {
    if (sortKey === key) {
      sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
    } else {
      sortKey = key;
      sortDirection = key === 'name' ? 'asc' : 'desc';
    }
  }

  function getSortIcon(key: SortKey): string {
    if (sortKey !== key) return '↕';
    return sortDirection === 'asc' ? '↑' : '↓';
  }

  function getStatusIcon(status: string): string {
    const s = status.toLowerCase();
    if (s === 'running') return '●';
    if (s === 'sleeping' || s === 'idle') return '○';
    if (s === 'zombie' || s === 'defunct') return '☠';
    if (s === 'stopped') return '■';
    return '?';
  }

  function getStatusColor(status: string): string {
    const s = status.toLowerCase();
    if (s === 'running') return 'var(--success)';
    if (s === 'sleeping' || s === 'idle') return 'var(--text-muted)';
    if (s === 'zombie' || s === 'defunct') return 'var(--danger)';
    if (s === 'stopped') return 'var(--warning)';
    return 'var(--text-secondary)';
  }

  function handleKill(process: ProcessInfo) {
    selectedProcess = process;
    showKillModal = true;
    killError = null;
  }

  async function confirmKill() {
    if (!selectedProcess) return;

    killing = true;
    killError = null;

    try {
      await killProcess(selectedProcess.pid);
      showKillModal = false;
      selectedProcess = null;
    } catch (err) {
      killError = err instanceof Error ? err.message : 'Failed to kill process';
    } finally {
      killing = false;
    }
  }

  function cancelKill() {
    showKillModal = false;
    selectedProcess = null;
    killError = null;
  }

  const filteredProcesses = $derived(
    $processes.filter(p =>
      p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      p.pid.toString().includes(searchQuery) ||
      p.user.toLowerCase().includes(searchQuery.toLowerCase())
    )
  );

  const sortedProcesses = $derived(
    [...filteredProcesses].sort((a, b) => {
      let comparison = 0;

      switch (sortKey) {
        case 'pid':
          comparison = a.pid - b.pid;
          break;
        case 'name':
          comparison = a.name.localeCompare(b.name);
          break;
        case 'cpu_percent':
          comparison = a.cpu_percent - b.cpu_percent;
          break;
        case 'memory_bytes':
          comparison = a.memory_bytes - b.memory_bytes;
          break;
        case 'status':
          comparison = a.status.localeCompare(b.status);
          break;
        case 'runtime_seconds':
          comparison = a.runtime_seconds - b.runtime_seconds;
          break;
      }

      return sortDirection === 'asc' ? comparison : -comparison;
    })
  );

  const displayedProcesses = $derived(
    showAll ? sortedProcesses : sortedProcesses.slice(0, DISPLAY_LIMIT)
  );
</script>

<div class="process-table-container">
  <div class="table-header">
    <h3 class="table-title">
      <svg class="title-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="2" y="3" width="20" height="14" rx="2" />
        <line x1="8" y1="21" x2="16" y2="21" />
        <line x1="12" y1="17" x2="12" y2="21" />
      </svg>
      Processes
      <span class="process-count">({filteredProcesses.length})</span>
    </h3>

    <div class="table-actions">
      <div class="search-box">
        <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8" />
          <line x1="21" y1="21" x2="16.65" y2="16.65" />
        </svg>
        <input
          type="text"
          placeholder="Search..."
          bind:value={searchQuery}
          class="search-input"
        />
        {#if searchQuery}
          <button class="clear-btn" onclick={() => searchQuery = ''}>
            ×
          </button>
        {/if}
      </div>
    </div>
  </div>

  <div class="table-wrapper">
    <table class="process-table">
      <thead>
        <tr>
          <th class="sortable" onclick={() => toggleSort('pid')}>
            PID {getSortIcon('pid')}
          </th>
          <th class="sortable" onclick={() => toggleSort('name')}>
            Name {getSortIcon('name')}
          </th>
          <th class="sortable" onclick={() => toggleSort('cpu_percent')}>
            CPU {getSortIcon('cpu_percent')}
          </th>
          <th class="sortable" onclick={() => toggleSort('memory_bytes')}>
            Memory {getSortIcon('memory_bytes')}
          </th>
          <th class="sortable" onclick={() => toggleSort('status')}>
            Status {getSortIcon('status')}
          </th>
          <th class="sortable" onclick={() => toggleSort('runtime_seconds')}>
            Age {getSortIcon('runtime_seconds')}
          </th>
          <th class="actions-col">Action</th>
        </tr>
      </thead>
      <tbody>
        {#each displayedProcesses as process (process.pid)}
          <tr
            class:stale={process.is_stale}
            class:zombie={process.is_zombie}
          >
            <td class="pid-cell">{process.pid}</td>
            <td class="name-cell">
              <span class="process-name" title={process.name}>
                {truncate(process.name, 25)}
              </span>
              {#if process.is_stale}
                <span class="badge stale-badge" title="Stale process">⚠</span>
              {/if}
              {#if process.is_zombie}
                <span class="badge zombie-badge" title="Zombie process">☠</span>
              {/if}
            </td>
            <td class="numeric-cell">{formatPercent(process.cpu_percent)}</td>
            <td class="numeric-cell">
              {formatBytes(process.memory_bytes)}
              <span class="mem-percent">({formatPercent(process.memory_percent)})</span>
            </td>
            <td>
              <span class="status-badge" style:color={getStatusColor(process.status)}>
                {getStatusIcon(process.status)} {process.status}
              </span>
            </td>
            <td class="numeric-cell">{formatDuration(process.runtime_seconds)}</td>
            <td class="actions-cell">
              <button
                class="kill-btn"
                onclick={() => handleKill(process)}
                title="Kill process"
                aria-label="Kill process {process.name}"
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="3 6 5 6 21 6" />
                  <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                </svg>
              </button>
            </td>
          </tr>
        {:else}
          <tr>
            <td colspan="7" class="empty-cell">
              {searchQuery ? 'No processes match your search' : 'No processes available'}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

  {#if filteredProcesses.length > DISPLAY_LIMIT}
    <div class="table-footer">
      <button class="show-more-btn" onclick={() => showAll = !showAll}>
        {showAll ? 'Show Less' : `Show All (${filteredProcesses.length})`}
      </button>
    </div>
  {/if}
</div>

<ConfirmModal
  isOpen={showKillModal}
  title="Kill Process"
  message={selectedProcess
    ? `Are you sure you want to terminate "${selectedProcess.name}" (PID: ${selectedProcess.pid})?\n\nThis action cannot be undone.`
    : ''}
  confirmText={killing ? 'Killing...' : 'Kill Process'}
  cancelText="Cancel"
  variant="danger"
  onConfirm={confirmKill}
  onCancel={cancelKill}
/>

{#if killError}
  <div class="error-toast animate-fade-in">
    <span>{killError}</span>
    <button onclick={() => killError = null}>×</button>
  </div>
{/if}

<style>
  .process-table-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-height: 0;
    background: var(--bg-surface);
  }

  .table-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 10px;
    border-bottom: 1px solid var(--border-color);
    gap: 8px;
    flex-shrink: 0;
  }

  .table-title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--font-sm);
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .title-icon {
    width: 14px;
    height: 14px;
    color: var(--accent);
  }

  .process-count {
    color: var(--text-muted);
    font-weight: 400;
  }

  .table-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .search-box {
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-icon {
    position: absolute;
    left: 8px;
    width: 12px;
    height: 12px;
    color: var(--text-muted);
  }

  .search-input {
    background: var(--bg-elevated);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    padding: 4px 8px 4px 26px;
    color: var(--text-primary);
    font-size: var(--font-xs);
    width: 140px;
    transition: all var(--transition-fast);
  }

  .search-input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .search-input::placeholder {
    color: var(--text-muted);
  }

  .clear-btn {
    position: absolute;
    right: 5px;
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 14px;
    line-height: 1;
    cursor: pointer;
    padding: 0;
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .clear-btn:hover {
    color: var(--text-primary);
  }

  .table-wrapper {
    flex: 1;
    overflow: auto;
    min-height: 0;
  }

  .process-table {
    width: 100%;
    font-size: var(--font-xs);
  }

  thead {
    position: sticky;
    top: 0;
    z-index: 10;
  }

  th {
    background: var(--bg-elevated);
    color: var(--text-secondary);
    font-weight: 500;
    text-align: left;
    padding: 5px 8px;
    border-bottom: 1px solid var(--border-color);
    white-space: nowrap;
    user-select: none;
    font-size: var(--font-xs);
  }

  th.sortable {
    cursor: pointer;
    transition: color var(--transition-fast);
  }

  th.sortable:hover {
    color: var(--text-primary);
  }

  td {
    padding: 3px 8px;
    border-bottom: 1px solid var(--border-color);
    color: var(--text-primary);
  }

  tbody tr {
    transition: background var(--transition-fast);
  }

  tbody tr:hover {
    background: var(--bg-elevated);
  }

  tbody tr.stale {
    background: rgba(245, 158, 11, 0.05);
  }

  tbody tr.stale:hover {
    background: rgba(245, 158, 11, 0.1);
  }

  tbody tr.zombie {
    background: rgba(239, 68, 68, 0.05);
  }

  tbody tr.zombie:hover {
    background: rgba(239, 68, 68, 0.1);
  }

  .pid-cell {
    font-family: var(--font-mono);
    color: var(--text-muted);
  }

  .name-cell {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .process-name {
    font-weight: 500;
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .badge {
    font-size: 10px;
    padding: 0 3px;
    border-radius: var(--radius-sm);
  }

  .stale-badge {
    background: rgba(245, 158, 11, 0.2);
  }

  .zombie-badge {
    background: rgba(239, 68, 68, 0.2);
  }

  .numeric-cell {
    text-align: right;
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }

  .mem-percent {
    color: var(--text-muted);
    font-size: 10px;
  }

  .status-badge {
    font-size: var(--font-xs);
    font-weight: 500;
  }

  .actions-col {
    width: 40px;
  }

  .actions-cell {
    text-align: center;
  }

  .kill-btn {
    background: transparent;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    padding: 2px;
    color: var(--text-muted);
    cursor: pointer;
    transition: all var(--transition-fast);
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .kill-btn svg {
    width: 12px;
    height: 12px;
  }

  .kill-btn:hover {
    background: rgba(239, 68, 68, 0.1);
    border-color: var(--danger);
    color: var(--danger);
  }

  .empty-cell {
    text-align: center;
    color: var(--text-muted);
    padding: 40px 16px !important;
  }

  .table-footer {
    padding: 6px 10px;
    border-top: 1px solid var(--border-color);
    text-align: center;
    flex-shrink: 0;
  }

  .show-more-btn {
    background: var(--bg-elevated);
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    padding: 4px 12px;
    border-radius: var(--radius-sm);
    font-size: var(--font-xs);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .show-more-btn:hover {
    background: var(--bg-surface-hover);
    color: var(--text-primary);
  }

  .error-toast {
    position: fixed;
    bottom: var(--space-4);
    right: var(--space-4);
    background: var(--danger);
    color: white;
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    gap: var(--space-3);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    z-index: 1000;
  }

  .error-toast button {
    background: none;
    border: none;
    color: white;
    font-size: 1.25rem;
    cursor: pointer;
    line-height: 1;
  }

  @media (max-width: 768px) {
    .table-header {
      flex-direction: column;
      align-items: stretch;
    }

    .search-input {
      width: 100%;
    }

    .process-name {
      max-width: 80px;
    }

    .mem-percent {
      display: none;
    }
  }
</style>

<script lang="ts">
  import { allProcesses } from '$lib/stores/metrics';
  import { formatBytes, formatPercent, formatDuration, truncate } from '$lib/utils/format';
  import type { ProcessInfo } from '$lib/types';
  import {
    paused,
    effectiveProcesses,
    togglePause,
    selectedPids,
    toggleSelect,
    clearSelection,
    selectAll,
    pinnedPids,
    togglePin,
    cpuHistory,
    visibleColumns,
    toggleColumn,
    cpuThreshold,
    memoryThreshold,
    detailProcess,
    showDetailDrawer,
    openProcessDetail,
    closeProcessDetail
  } from '$lib/stores/processState';
  import ProcessActions from './ProcessActions.svelte';
  import ProcessDetailDrawer from './ProcessDetailDrawer.svelte';
  import ColumnPicker from './ColumnPicker.svelte';
  import CpuSparkline from './CpuSparkline.svelte';
  import SignalPicker from './SignalPicker.svelte';

  type SortKey = 'pid' | 'name' | 'cpu_percent' | 'memory_bytes' | 'status' | 'runtime_seconds' | 'user' | 'command';
  type SortDirection = 'asc' | 'desc';
  type ProcessFilter = 'all' | 'active' | 'idle' | 'stale' | 'zombie';

  interface ProcessTreeNode {
    process: ProcessInfo;
    children: ProcessTreeNode[];
    depth: number;
    subtreeMaxCpu: number;
    subtreeTotalMemory: number;
  }

  /* ── Column definitions (Feature 6) ─────────────────── */
  const COLUMNS: { key: string; label: string }[] = [
    { key: 'pid', label: 'PID' },
    { key: 'name', label: 'Name' },
    { key: 'cpu_percent', label: 'CPU' },
    { key: 'memory_bytes', label: 'Memory' },
    { key: 'status', label: 'Status' },
    { key: 'runtime_seconds', label: 'Age' },
    { key: 'user', label: 'User' },
    { key: 'command', label: 'Command' },
  ];

  const DISPLAY_LIMIT = 500;

  /* ── State ──────────────────────────────────────────── */
  let sortKey = $state<SortKey>('cpu_percent');
  let sortDirection = $state<SortDirection>('desc');
  let searchQuery = $state('');
  let activeFilter = $state<ProcessFilter>('all');
  let showAll = $state(false);
  let selectedProcess: ProcessInfo | null = $state(null);
  let showKillModal = $state(false);
  let killing = $state(false);
  let killError = $state<string | null>(null);
  let signalToSend = $state('SIGTERM');
  let expandedPids = $state(new Set<number>());
  let preSearchExpanded = $state(new Set<number>());
  let lastSearchQuery = $state('');

  /* ── Refs ────────────────────────────────────────────── */
  let killDialogRef: HTMLDialogElement | null = $state(null);
  let selectAllRef: HTMLInputElement | null = $state(null);

  /* ── Pure helpers ───────────────────────────────────── */
  function matchesFilter(p: ProcessInfo): boolean {
    switch (activeFilter) {
      case 'active': return p.cpu_percent > 0 || p.status === 'running';
      case 'idle': return p.status === 'idle' && p.cpu_percent === 0;
      case 'stale': return p.is_stale;
      case 'zombie': return p.is_zombie;
      default: return true;
    }
  }

  function toggleSort(key: SortKey) {
    if (sortKey === key) {
      sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
    } else {
      sortKey = key;
      sortDirection = key === 'name' || key === 'user' || key === 'command' ? 'asc' : 'desc';
    }
  }

  function getSortIcon(key: SortKey): string {
    if (sortKey !== key) return '↕';
    return sortDirection === 'asc' ? '↑' : '↓';
  }

  function getStatusIcon(status: string): string {
    const s = status.toLowerCase();
    if (s === 'running') return '●';
    if (s === 'idle') return '○';
    if (s === 'zombie' || s === 'defunct' || s === 'dead') return '☠';
    if (s === 'stopped') return '■';
    if (s === 'blocked') return '⏸';
    return '○';
  }

  function getStatusColor(status: string): string {
    const s = status.toLowerCase();
    if (s === 'running') return 'var(--success)';
    if (s === 'idle') return 'var(--text-muted)';
    if (s === 'zombie' || s === 'defunct' || s === 'dead') return 'var(--danger)';
    if (s === 'stopped') return 'var(--warning)';
    if (s === 'blocked') return 'var(--warning)';
    return 'var(--text-muted)';
  }

  function toggleExpand(pid: number) {
    const next = new Set(expandedPids);
    if (next.has(pid)) {
      next.delete(pid);
    } else {
      next.add(pid);
    }
    expandedPids = next;
  }

  function expandAll() {
    expandedPids = new Set(filteredByFilter.map(p => p.pid));
  }

  function collapseAll() {
    expandedPids = new Set();
  }

  /* ── Kill handlers (Feature 8: signal-aware) ────────── */
  function handleKill(process: ProcessInfo) {
    selectedProcess = process;
    signalToSend = 'SIGTERM';
    showKillModal = true;
    killError = null;
  }

  async function confirmKill() {
    if (!selectedProcess) return;
    killing = true;
    killError = null;
    try {
      const res = await fetch('/api/v1/actions/kill', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ pid: selectedProcess.pid, signal: signalToSend })
      });
      if (!res.ok) {
        const error = await res.text();
        throw new Error(error || `Failed to kill process ${selectedProcess.pid}`);
      }
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

  function handleKillBackdropClick(event: MouseEvent) {
    if (event.target === killDialogRef) {
      cancelKill();
    }
  }

  /* ── Multi-select handlers (Feature 3) ──────────────── */
  function handleSelectAll() {
    const visiblePids = displayedProcesses.map(n => n.process.pid);
    const allSel = visiblePids.length > 0 && visiblePids.every(pid => $selectedPids.has(pid));
    if (allSel) {
      clearSelection();
    } else {
      selectAll(visiblePids);
    }
  }

  async function handleBatchKill() {
    try {
      await fetch('/api/v1/actions/kill-batch', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ pids: Array.from($selectedPids) })
      });
    } catch (err) {
      killError = err instanceof Error ? err.message : 'Batch kill failed';
    }
    clearSelection();
  }

  async function handleBatchSignal(signal: string) {
    try {
      await fetch('/api/v1/actions/kill-batch', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ pids: Array.from($selectedPids), signal })
      });
    } catch (err) {
      killError = err instanceof Error ? err.message : 'Batch signal failed';
    }
    clearSelection();
  }

  /* ── Derived values ─────────────────────────────────── */

  // Feature 1: Use effectiveProcesses (paused snapshot or live)
  const filteredByFilter = $derived(
    activeFilter === 'all' ? $effectiveProcesses : $effectiveProcesses.filter(matchesFilter)
  );

  // Feature 2: Stable sort with PID tiebreaker
  const processTree = $derived.by(() => {
    const byPpid = new Map<number, ProcessInfo[]>();
    const pidSet = new Set<number>();

    for (const p of filteredByFilter) {
      pidSet.add(p.pid);
      const parentId = p.ppid ?? 0;
      if (!byPpid.has(parentId)) {
        byPpid.set(parentId, []);
      }
      byPpid.get(parentId)!.push(p);
    }

    const sortFn = (a: ProcessInfo, b: ProcessInfo) => {
      let comparison = 0;
      switch (sortKey) {
        case 'pid': comparison = a.pid - b.pid; break;
        case 'name': comparison = a.name.localeCompare(b.name); break;
        case 'cpu_percent': comparison = a.cpu_percent - b.cpu_percent; break;
        case 'memory_bytes': comparison = a.memory_bytes - b.memory_bytes; break;
        case 'status': comparison = a.status.localeCompare(b.status); break;
        case 'runtime_seconds': comparison = a.runtime_seconds - b.runtime_seconds; break;
        case 'user': comparison = a.user.localeCompare(b.user); break;
        case 'command': comparison = a.command.localeCompare(b.command); break;
      }
      // Stable sort: PID as secondary tiebreaker
      if (comparison === 0) comparison = a.pid - b.pid;
      return sortDirection === 'asc' ? comparison : -comparison;
    };

    function buildNode(p: ProcessInfo, depth: number): ProcessTreeNode {
      const childrenInfos = byPpid.get(p.pid) || [];
      childrenInfos.sort(sortFn);
      const children = childrenInfos.map(child => buildNode(child, depth + 1));

      let subtreeMaxCpu = p.cpu_percent;
      let subtreeTotalMemory = p.memory_bytes;
      for (const child of children) {
        if (child.subtreeMaxCpu > subtreeMaxCpu) subtreeMaxCpu = child.subtreeMaxCpu;
        subtreeTotalMemory += child.subtreeTotalMemory;
      }

      return { process: p, children, depth, subtreeMaxCpu, subtreeTotalMemory };
    }

    const roots: ProcessInfo[] = [];
    for (const p of filteredByFilter) {
      if (p.ppid == null || p.ppid === 0 || !pidSet.has(p.ppid)) {
        roots.push(p);
      }
    }

    const rootNodes = roots.map(root => buildNode(root, 0));

    const treeSortFn = (a: ProcessTreeNode, b: ProcessTreeNode) => {
      let comparison = 0;
      switch (sortKey) {
        case 'cpu_percent': comparison = a.subtreeMaxCpu - b.subtreeMaxCpu; break;
        case 'memory_bytes': comparison = a.subtreeTotalMemory - b.subtreeTotalMemory; break;
        default: comparison = sortFn(a.process, b.process); return comparison;
      }
      return sortDirection === 'asc' ? comparison : -comparison;
    };

    rootNodes.sort(treeSortFn);
    return rootNodes;
  });

  // Feature 4: Pinned processes rendered first
  const flattenedProcesses = $derived.by(() => {
    const result: ProcessTreeNode[] = [];
    const isSearchActive = !!searchQuery;
    let matches: Set<number> | null = null;

    if (isSearchActive) {
      const q = searchQuery.toLowerCase();
      matches = new Set();
      for (const p of filteredByFilter) {
        if (p.name.toLowerCase().includes(q) || p.pid.toString().includes(q) || p.user.toLowerCase().includes(q)) {
          matches.add(p.pid);
        }
      }
    }

    function flatten(nodes: ProcessTreeNode[]) {
      for (const node of nodes) {
        let shouldRender = true;
        if (isSearchActive && matches) {
           shouldRender = matches.has(node.process.pid) || expandedPids.has(node.process.pid);
        }

        if (shouldRender) {
          result.push(node);
          if (expandedPids.has(node.process.pid) && node.children.length > 0) {
            flatten(node.children);
          }
        }
      }
    }
    flatten(processTree);

    // Pinned processes float to top
    const pinned = result.filter(n => $pinnedPids.has(n.process.pid));
    const regular = result.filter(n => !$pinnedPids.has(n.process.pid));
    return [...pinned, ...regular];
  });

  const displayedProcesses = $derived(
    showAll ? flattenedProcesses : flattenedProcesses.slice(0, DISPLAY_LIMIT)
  );

  // Feature 3: Select-all state
  const allVisibleSelected = $derived(
    displayedProcesses.length > 0 && displayedProcesses.every(n => $selectedPids.has(n.process.pid))
  );

  const someVisibleSelected = $derived(
    displayedProcesses.length > 0 &&
    displayedProcesses.some(n => $selectedPids.has(n.process.pid)) &&
    !allVisibleSelected
  );

  // Feature 6: Dynamic colspan
  const totalColumns = $derived($visibleColumns.size + 2);

  /* ── Effects ────────────────────────────────────────── */

  // Search auto-expand ancestors
  $effect(() => {
    if (searchQuery !== lastSearchQuery) {
      if (searchQuery && !lastSearchQuery) {
        preSearchExpanded = new Set(expandedPids);
      }

      if (searchQuery) {
        const q = searchQuery.toLowerCase();
        const matches = new Set<number>();
        const parentMap = new Map<number, number>();

        for (const p of filteredByFilter) {
          if (p.ppid != null) parentMap.set(p.pid, p.ppid);
          if (
            p.name.toLowerCase().includes(q) ||
            p.pid.toString().includes(q) ||
            p.user.toLowerCase().includes(q)
          ) {
            matches.add(p.pid);
          }
        }

        const newExpanded = new Set<number>();
        for (const pid of matches) {
          let curr = parentMap.get(pid);
          while (curr != null && curr !== 0) {
            newExpanded.add(curr);
            curr = parentMap.get(curr);
          }
        }

        expandedPids = newExpanded;
      } else if (!searchQuery && lastSearchQuery) {
        expandedPids = new Set(preSearchExpanded);
      }
      lastSearchQuery = searchQuery;
    }
  });

  // Kill dialog open/close
  $effect(() => {
    if (showKillModal && killDialogRef) {
      killDialogRef.showModal();
    } else if (!showKillModal && killDialogRef?.open) {
      killDialogRef.close();
    }
  });

  // Select-all indeterminate state
  $effect(() => {
    if (selectAllRef) {
      selectAllRef.indeterminate = someVisibleSelected;
    }
  });
</script>

<div class="process-table-container" class:paused={$paused}>
  <div class="table-header">
    <h3 class="table-title">
      <svg class="title-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="2" y="3" width="20" height="14" rx="2" />
        <line x1="8" y1="21" x2="16" y2="21" />
        <line x1="12" y1="17" x2="12" y2="21" />
      </svg>
      Processes
      <button
        class="pause-btn"
        class:active={$paused}
        onclick={togglePause}
        title={$paused ? 'Resume updates' : 'Pause updates'}
      >
        {$paused ? '▶' : '⏸'}
        {#if $paused}<span class="pause-label">Paused</span>{/if}
      </button>
      <span class="process-count">({flattenedProcesses.length} / {$allProcesses.length})</span>
    </h3>

    <div class="table-actions">
      <div class="filter-group">
        {#each [
          { key: 'all', label: 'All' },
          { key: 'active', label: 'Active' },
          { key: 'idle', label: 'Idle' },
          { key: 'stale', label: 'Stale' },
          { key: 'zombie', label: 'Zombie' },
        ] as filter}
          <button
            class="filter-btn"
            class:active={activeFilter === filter.key}
            onclick={() => activeFilter = filter.key as ProcessFilter}
          >
            {filter.label}
          </button>
        {/each}
      </div>
      <button class="header-btn" onclick={expandAll}>Expand All</button>
      <button class="header-btn" onclick={collapseAll}>Collapse All</button>
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
      <ColumnPicker columns={COLUMNS} visible={$visibleColumns} onChange={toggleColumn} />
    </div>
  </div>

  <div class="table-wrapper">
    <table class="process-table">
      <thead>
        <tr>
          <th class="select-col">
            <input
              bind:this={selectAllRef}
              type="checkbox"
              checked={allVisibleSelected}
              onchange={handleSelectAll}
              title="Select all visible"
            />
          </th>
          {#if $visibleColumns.has('pid')}
            <th class="sortable" onclick={() => toggleSort('pid')}>
              PID {getSortIcon('pid')}
            </th>
          {/if}
          {#if $visibleColumns.has('name')}
            <th class="sortable" onclick={() => toggleSort('name')}>
              Name {getSortIcon('name')}
            </th>
          {/if}
          {#if $visibleColumns.has('cpu_percent')}
            <th class="sortable" onclick={() => toggleSort('cpu_percent')}>
              CPU {getSortIcon('cpu_percent')}
            </th>
          {/if}
          {#if $visibleColumns.has('memory_bytes')}
            <th class="sortable" onclick={() => toggleSort('memory_bytes')}>
              Memory {getSortIcon('memory_bytes')}
            </th>
          {/if}
          {#if $visibleColumns.has('status')}
            <th class="sortable" onclick={() => toggleSort('status')}>
              Status {getSortIcon('status')}
            </th>
          {/if}
          {#if $visibleColumns.has('runtime_seconds')}
            <th class="sortable" onclick={() => toggleSort('runtime_seconds')}>
              Age {getSortIcon('runtime_seconds')}
            </th>
          {/if}
          {#if $visibleColumns.has('user')}
            <th class="sortable" onclick={() => toggleSort('user')}>
              User {getSortIcon('user')}
            </th>
          {/if}
          {#if $visibleColumns.has('command')}
            <th class="sortable" onclick={() => toggleSort('command')}>
              Command {getSortIcon('command')}
            </th>
          {/if}
          <th class="actions-col">Action</th>
        </tr>
      </thead>
      <tbody>
        {#each displayedProcesses as node (node.process.pid)}
          <tr
            class:stale={node.process.is_stale}
            class:zombie={node.process.is_zombie}
            class:high-cpu={node.process.cpu_percent > $cpuThreshold}
            class:high-memory={node.process.memory_percent > $memoryThreshold}
            class:selected={$selectedPids.has(node.process.pid)}
            class:pinned={$pinnedPids.has(node.process.pid)}
            ondblclick={() => openProcessDetail(node.process)}
          >
            <td class="select-cell">
              <input
                type="checkbox"
                checked={$selectedPids.has(node.process.pid)}
                onchange={() => toggleSelect(node.process.pid)}
              />
            </td>

            {#if $visibleColumns.has('pid')}
              <td class="pid-cell">{node.process.pid}</td>
            {/if}

            {#if $visibleColumns.has('name')}
              <td class="name-cell" style="padding-left: {8 + node.depth * 16}px;">
                {#if node.depth > 0}
                  <div class="tree-line" style="left: {12 + (node.depth - 1) * 16}px;"></div>
                {/if}

                {#if node.children.length > 0}
                  <button class="chevron-btn" onclick={() => toggleExpand(node.process.pid)}>
                    {expandedPids.has(node.process.pid) ? '▼' : '▶'}
                  </button>
                {:else}
                  <span class="chevron-spacer"></span>
                {/if}

                <span class="process-name" title={node.process.name}>
                  {truncate(node.process.name, 25)}
                </span>

                {#if node.children.length > 0 && !expandedPids.has(node.process.pid)}
                  <span class="child-count">({node.children.length})</span>
                {/if}

                {#if node.process.is_stale}
                  <span class="badge stale-badge" title="Stale process">⚠</span>
                {/if}
                {#if node.process.is_zombie}
                  <span class="badge zombie-badge" title="Zombie process">☠</span>
                {/if}
              </td>
            {/if}

            {#if $visibleColumns.has('cpu_percent')}
              <td class="numeric-cell cpu-cell">
                {formatPercent(node.process.cpu_percent)}
                {#if $cpuHistory.get(node.process.pid)?.length}
                  <span class="sparkline-wrapper">
                    <CpuSparkline values={$cpuHistory.get(node.process.pid) ?? []} width={50} height={14} />
                  </span>
                {/if}
              </td>
            {/if}

            {#if $visibleColumns.has('memory_bytes')}
              <td class="numeric-cell">
                {formatBytes(node.process.memory_bytes)}
                <span class="mem-percent">({formatPercent(node.process.memory_percent)})</span>
              </td>
            {/if}

            {#if $visibleColumns.has('status')}
              <td>
                <span class="status-badge" style:color={getStatusColor(node.process.status)}>
                  {getStatusIcon(node.process.status)} {node.process.status}
                </span>
              </td>
            {/if}

            {#if $visibleColumns.has('runtime_seconds')}
              <td class="numeric-cell">{formatDuration(node.process.runtime_seconds)}</td>
            {/if}

            {#if $visibleColumns.has('user')}
              <td class="user-cell">{node.process.user}</td>
            {/if}

            {#if $visibleColumns.has('command')}
              <td class="command-cell" title={node.process.command}>
                {truncate(node.process.command, 40)}
              </td>
            {/if}

            <td class="actions-cell">
              <button
                class="pin-btn"
                class:active={$pinnedPids.has(node.process.pid)}
                onclick={() => togglePin(node.process.pid)}
                title={$pinnedPids.has(node.process.pid) ? 'Unpin process' : 'Pin process'}
              >
                📌
              </button>
              <button
                class="kill-btn"
                onclick={() => handleKill(node.process)}
                title="Kill process"
                aria-label="Kill process {node.process.name}"
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
            <td colspan={totalColumns} class="empty-cell">
              {searchQuery ? 'No processes match your search' : 'No processes available'}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

  {#if flattenedProcesses.length > DISPLAY_LIMIT}
    <div class="table-footer">
      <button class="show-more-btn" onclick={() => showAll = !showAll}>
        {showAll ? 'Show Less' : `Show All (${flattenedProcesses.length})`}
      </button>
    </div>
  {/if}
</div>

{#if $selectedPids.size > 0}
  <ProcessActions
    selectedCount={$selectedPids.size}
    onKill={handleBatchKill}
    onSignal={handleBatchSignal}
    onClear={clearSelection}
  />
{/if}

<dialog
  bind:this={killDialogRef}
  class="kill-dialog"
  onclick={handleKillBackdropClick}
  oncancel={(e) => { e.preventDefault(); cancelKill(); }}
>
  {#if selectedProcess}
    <div class="kill-dialog-content">
      <header class="kill-dialog-header">
        <h2 class="kill-dialog-title">Kill Process</h2>
      </header>
      <div class="kill-dialog-body">
        <p class="kill-dialog-message">
          Are you sure you want to terminate "<strong>{selectedProcess.name}</strong>" (PID: {selectedProcess.pid})?
        </p>
        <p class="kill-dialog-warning">This action cannot be undone.</p>
        <div class="signal-picker-section">
          <SignalPicker selected={signalToSend} onSelect={(s) => signalToSend = s} compact />
        </div>
        {#if killError}
          <p class="kill-error">{killError}</p>
        {/if}
      </div>
      <footer class="kill-dialog-footer">
        <button class="dialog-btn dialog-btn-cancel" onclick={cancelKill}>
          Cancel
        </button>
        <button class="dialog-btn dialog-btn-confirm" onclick={confirmKill} disabled={killing}>
          {killing ? 'Killing...' : 'Kill Process'}
        </button>
      </footer>
    </div>
  {/if}
</dialog>

<ProcessDetailDrawer
  process={$detailProcess}
  isOpen={$showDetailDrawer}
  cpuHistory={$detailProcess ? $cpuHistory.get($detailProcess.pid) ?? [] : []}
  onClose={closeProcessDetail}
  onKill={(p) => handleKill(p)}
/>

{#if killError}
  <div class="error-toast animate-fade-in">
    <span>{killError}</span>
    <button onclick={() => killError = null}>×</button>
  </div>
{/if}

<style>
  /* ── Container ──────────────────────────────────────── */
  .process-table-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-height: 0;
    background: var(--bg-surface);
    transition: box-shadow var(--transition-fast);
  }

  .process-table-container.paused {
    box-shadow: inset 0 0 0 1px rgba(245, 158, 11, 0.3), 0 0 8px rgba(245, 158, 11, 0.08);
  }

  /* ── Header ─────────────────────────────────────────── */
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

  /* ── Pause button (Feature 1) ───────────────────────── */
  .pause-btn {
    background: none;
    border: 1px solid var(--border-color);
    color: var(--text-muted);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-size: var(--font-xs);
    cursor: pointer;
    transition: all var(--transition-fast);
    display: inline-flex;
    align-items: center;
    gap: 4px;
    line-height: 1;
  }

  .pause-btn:hover {
    color: var(--text-primary);
    border-color: var(--text-muted);
  }

  .pause-btn.active {
    color: var(--warning);
    border-color: var(--warning);
    background: rgba(245, 158, 11, 0.1);
  }

  .pause-label {
    font-size: 10px;
    font-weight: 500;
  }

  /* ── Actions bar ────────────────────────────────────── */
  .table-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .filter-group {
    display: flex;
    gap: 1px;
    background: var(--border-color);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .filter-btn {
    background: var(--bg-elevated);
    border: none;
    color: var(--text-muted);
    padding: 3px 8px;
    font-size: var(--font-xs);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .filter-btn:hover {
    color: var(--text-primary);
    background: var(--bg-surface-hover, rgba(255, 255, 255, 0.05));
  }

  .filter-btn.active {
    background: var(--accent);
    color: white;
  }

  .header-btn {
    background: var(--bg-elevated);
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    padding: 3px 8px;
    border-radius: var(--radius-sm);
    font-size: var(--font-xs);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .header-btn:hover {
    background: var(--bg-surface-hover, rgba(255, 255, 255, 0.05));
    color: var(--text-primary);
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

  /* ── Table ──────────────────────────────────────────── */
  .table-wrapper {
    flex: 1;
    overflow: auto;
    min-height: 0;
  }

  .process-table {
    width: 100%;
    font-size: var(--font-xs);
    border-collapse: collapse;
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

  /* ── Row state classes ──────────────────────────────── */
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

  /* Feature 3: Selected row */
  tbody tr.selected {
    background: rgba(59, 130, 246, 0.08);
  }

  tbody tr.selected:hover {
    background: rgba(59, 130, 246, 0.12);
  }

  /* Feature 4: Pinned row */
  tbody tr.pinned {
    border-left: 2px solid var(--accent);
  }

  /* Feature 9: Threshold highlighting */
  tbody tr.high-cpu {
    background: rgba(239, 68, 68, 0.06);
  }

  tbody tr.high-memory {
    background: rgba(139, 92, 246, 0.06);
  }

  tbody tr.high-cpu.high-memory {
    background: rgba(239, 68, 68, 0.04);
  }

  /* ── Select column (Feature 3) ──────────────────────── */
  .select-col {
    width: 28px;
    text-align: center;
  }

  .select-cell {
    text-align: center;
  }

  .select-cell input[type="checkbox"],
  .select-col input[type="checkbox"] {
    cursor: pointer;
    accent-color: var(--accent);
  }

  /* ── Cell styles ────────────────────────────────────── */
  .pid-cell {
    font-family: var(--font-mono);
    color: var(--text-muted);
  }

  .name-cell {
    position: relative;
    display: flex;
    align-items: center;
    gap: 4px;
    min-height: 24px;
  }

  .tree-line {
    position: absolute;
    top: -10px;
    bottom: 50%;
    width: 8px;
    border-left: 1px solid var(--border-color);
    border-bottom: 1px solid var(--border-color);
    opacity: 0.5;
    pointer-events: none;
  }

  .chevron-btn {
    background: none;
    border: none;
    color: var(--accent);
    font-size: var(--font-xs);
    cursor: pointer;
    padding: 0;
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .chevron-btn:hover {
    color: var(--text-primary);
  }

  .chevron-spacer {
    display: inline-block;
    width: 16px;
    height: 16px;
    flex-shrink: 0;
  }

  .child-count {
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: var(--font-xs);
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

  /* Feature 5: CPU cell with sparkline */
  .cpu-cell {
    white-space: nowrap;
  }

  .sparkline-wrapper {
    display: inline-block;
    vertical-align: middle;
    margin-left: 4px;
  }

  .mem-percent {
    color: var(--text-muted);
    font-size: 10px;
  }

  .status-badge {
    font-size: var(--font-xs);
    font-weight: 500;
  }

  /* Feature 6: User & command columns */
  .user-cell {
    color: var(--text-secondary);
    font-size: var(--font-xs);
  }

  .command-cell {
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: var(--font-xs);
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* ── Actions column ─────────────────────────────────── */
  .actions-col {
    width: 60px;
  }

  .actions-cell {
    text-align: center;
    white-space: nowrap;
  }

  /* Feature 4: Pin button */
  .pin-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 2px;
    font-size: 10px;
    line-height: 1;
    opacity: 0.4;
    transition: all var(--transition-fast);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    vertical-align: middle;
  }

  .pin-btn:hover {
    opacity: 1;
  }

  .pin-btn.active {
    opacity: 1;
    color: var(--accent);
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
    vertical-align: middle;
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

  /* ── Footer ─────────────────────────────────────────── */
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
    background: var(--bg-surface-hover, rgba(255, 255, 255, 0.05));
    color: var(--text-primary);
  }

  /* ── Kill dialog (Feature 8) ────────────────────────── */
  .kill-dialog {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    padding: 0;
    max-width: 420px;
    width: 90%;
    color: var(--text-primary);
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
  }

  .kill-dialog::backdrop {
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(4px);
  }

  .kill-dialog-content {
    padding: var(--space-6, 1.5rem);
  }

  .kill-dialog-header {
    margin-bottom: var(--space-4, 1rem);
  }

  .kill-dialog-title {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--danger);
    margin: 0;
  }

  .kill-dialog-body {
    margin-bottom: var(--space-6, 1.5rem);
  }

  .kill-dialog-message {
    color: var(--text-secondary);
    line-height: 1.6;
    margin: 0 0 var(--space-2, 0.5rem) 0;
  }

  .kill-dialog-warning {
    color: var(--text-muted);
    font-size: var(--font-xs);
    margin: 0 0 var(--space-4, 1rem) 0;
  }

  .signal-picker-section {
    margin-top: var(--space-3, 0.75rem);
  }

  .kill-error {
    color: var(--danger);
    font-size: var(--font-xs);
    margin-top: var(--space-3, 0.75rem);
  }

  .kill-dialog-footer {
    display: flex;
    gap: var(--space-3, 0.75rem);
    justify-content: flex-end;
  }

  .dialog-btn {
    padding: var(--space-2, 0.5rem) var(--space-4, 1rem);
    border-radius: var(--radius-md);
    font-size: 0.875rem;
    font-weight: 500;
    transition: all var(--transition-fast);
    border: 1px solid transparent;
    cursor: pointer;
  }

  .dialog-btn-cancel {
    background: transparent;
    border-color: var(--border-color);
    color: var(--text-secondary);
  }

  .dialog-btn-cancel:hover {
    background: var(--bg-surface-hover);
    color: var(--text-primary);
  }

  .dialog-btn-confirm {
    background: var(--danger);
    color: white;
  }

  .dialog-btn-confirm:hover {
    background: var(--danger-hover);
  }

  .dialog-btn-confirm:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  /* ── Error toast ────────────────────────────────────── */
  .error-toast {
    position: fixed;
    bottom: var(--space-4, 16px);
    right: var(--space-4, 16px);
    background: var(--danger);
    color: white;
    padding: var(--space-3, 12px) var(--space-4, 16px);
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    gap: var(--space-3, 12px);
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

  /* ── Responsive ─────────────────────────────────────── */
  @media (max-width: 768px) {
    .table-header {
      flex-direction: column;
      align-items: stretch;
    }

    .table-actions {
      flex-wrap: wrap;
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

    .command-cell {
      max-width: 100px;
    }
  }
</style>

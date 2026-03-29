import { writable, derived, get } from 'svelte/store';
import type { ProcessInfo } from '$lib/types';

export const CPU_HISTORY_LENGTH = 30;

export const allProcesses = writable<ProcessInfo[]>([]);

export const paused = writable(false);
export const pausedSnapshot = writable<ProcessInfo[]>([]);
export const pinnedPids = writable<Set<number>>(new Set());
export const selectedPids = writable<Set<number>>(new Set());
export const cpuHistory = writable<Map<number, number[]>>(new Map());
export const visibleColumns = writable<Set<string>>(
  new Set(['pid', 'name', 'cpu_percent', 'memory_bytes', 'status', 'runtime_seconds'])
);
export const cpuThreshold = writable(80);
export const memoryThreshold = writable(90);
export const detailProcess = writable<ProcessInfo | null>(null);
export const showDetailDrawer = writable(false);

export const effectiveProcesses = derived(
  [paused, pausedSnapshot, allProcesses],
  ([$paused, $pausedSnapshot, $allProcesses]) => {
    return $paused ? $pausedSnapshot : $allProcesses;
  }
);

export function togglePause(): void {
  const isPaused = get(paused);
  if (!isPaused) {
    pausedSnapshot.set(get(allProcesses));
  }
  paused.update(v => !v);
}

export function togglePin(pid: number): void {
  pinnedPids.update(pins => {
    const next = new Set(pins);
    if (next.has(pid)) {
      next.delete(pid);
    } else {
      next.add(pid);
    }
    return next;
  });
}

export function toggleSelect(pid: number): void {
  selectedPids.update(sel => {
    const next = new Set(sel);
    if (next.has(pid)) {
      next.delete(pid);
    } else {
      next.add(pid);
    }
    return next;
  });
}

export function selectAll(pids: number[]): void {
  selectedPids.set(new Set(pids));
}

export function clearSelection(): void {
  selectedPids.set(new Set());
}

export function toggleColumn(key: string): void {
  visibleColumns.update(cols => {
    const next = new Set(cols);
    if (next.has(key)) {
      if (next.size > 2) next.delete(key);
    } else {
      next.add(key);
    }
    return next;
  });
}

export function updateCpuHistory(processes: ProcessInfo[]): void {
  cpuHistory.update(history => {
    const next = new Map(history);
    const activePids = new Set<number>();

    for (const p of processes) {
      activePids.add(p.pid);
      const existing = next.get(p.pid) || [];
      const updated = [...existing, p.cpu_percent].slice(-CPU_HISTORY_LENGTH);
      next.set(p.pid, updated);
    }

    for (const pid of next.keys()) {
      if (!activePids.has(pid)) {
        next.delete(pid);
      }
    }

    return next;
  });
}

export function openProcessDetail(process: ProcessInfo): void {
  detailProcess.set(process);
  showDetailDrawer.set(true);
}

export function closeProcessDetail(): void {
  showDetailDrawer.set(false);
  detailProcess.set(null);
}

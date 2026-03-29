import { writable, derived } from 'svelte/store';
import { metrics } from './metrics';

export interface TimeSeriesPoint {
  timestamp: number;
  value: number;
}

const MAX_POINTS = 150;

export const cpuHistory = writable<TimeSeriesPoint[]>([]);
export const memoryHistory = writable<TimeSeriesPoint[]>([]);

function appendPoint(store: typeof cpuHistory, value: number): void {
  store.update(points => {
    const next = [...points, { timestamp: Date.now(), value }];
    return next.length > MAX_POINTS ? next.slice(-MAX_POINTS) : next;
  });
}

let unsubscribe: (() => void) | null = null;

export function startMetricsHistory(): void {
  if (unsubscribe) return;
  unsubscribe = metrics.subscribe($m => {
    if (!$m) return;
    appendPoint(cpuHistory, $m.cpu.overall_percent);
    appendPoint(memoryHistory, $m.memory.used_percent);
  });
}

export function stopMetricsHistory(): void {
  unsubscribe?.();
  unsubscribe = null;
}

export const latestCpu = derived(cpuHistory, $h => $h[$h.length - 1]?.value ?? 0);
export const latestMemory = derived(memoryHistory, $h => $h[$h.length - 1]?.value ?? 0);

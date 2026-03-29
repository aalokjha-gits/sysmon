import { writable, derived } from 'svelte/store';
import { metrics } from './metrics';

export interface TimeSeriesPoint {
  timestamp: number;
  value: number;
}

const MAX_POINTS = 150;

export const cpuHistory = writable<TimeSeriesPoint[]>([]);
export const memoryHistory = writable<TimeSeriesPoint[]>([]);
export const temperatureHistory = writable<TimeSeriesPoint[]>([]);
export const gpuHistory = writable<TimeSeriesPoint[]>([]);
export const coreHistory = writable<Map<number, TimeSeriesPoint[]>>(new Map());

function appendPoint(store: typeof cpuHistory, value: number): void {
  store.update(points => {
    const next = [...points, { timestamp: Date.now(), value }];
    return next.length > MAX_POINTS ? next.slice(-MAX_POINTS) : next;
  });
}

function appendCorePoints(cores: Array<{ usage_percent: number }>): void {
  const now = Date.now();
  coreHistory.update(map => {
    const next = new Map(map);
    for (let i = 0; i < cores.length; i++) {
      const existing = next.get(i) ?? [];
      const updated = [...existing, { timestamp: now, value: cores[i].usage_percent }];
      next.set(i, updated.length > MAX_POINTS ? updated.slice(-MAX_POINTS) : updated);
    }
    return next;
  });
}

let unsubscribe: (() => void) | null = null;

export function startMetricsHistory(): void {
  if (unsubscribe) return;
  unsubscribe = metrics.subscribe($m => {
    if (!$m) return;
    appendPoint(cpuHistory, $m.cpu.overall_percent);
    appendPoint(memoryHistory, $m.memory.used_percent);
    if ($m.cpu.cores?.length > 0) {
      appendCorePoints($m.cpu.cores);
    }

    const temps = $m.temperature?.sensors
      ?.map(s => s.temperature_celsius)
      .filter((t): t is number => t != null && t > 0 && t < 150) ?? [];
    if (temps.length > 0) {
      appendPoint(temperatureHistory, temps.reduce((a, b) => a + b, 0) / temps.length);
    }

    // GPU: utilization of first GPU
    const gpuUtil = $m.gpu?.gpus?.[0]?.utilization_percent;
    if (gpuUtil != null) {
      appendPoint(gpuHistory, gpuUtil);
    }
  });
}

export function stopMetricsHistory(): void {
  unsubscribe?.();
  unsubscribe = null;
}

export const latestCpu = derived(cpuHistory, $h => $h[$h.length - 1]?.value ?? 0);
export const latestMemory = derived(memoryHistory, $h => $h[$h.length - 1]?.value ?? 0);

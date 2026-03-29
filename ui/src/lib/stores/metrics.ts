import { writable, derived } from 'svelte/store';
import type {
  SystemMetrics,
  Alert,
  StaleResponse,
  KillResponse,
  CleanupResponse,
  HealthResponse,
  ContainersResponse,
  WsMessage
} from '$lib/types';

// Connection state
export const connected = writable(false);
export const connecting = writable(false);
export const lastUpdate = writable<Date | null>(null);
export const connectionError = writable<string | null>(null);

// Data stores
export const metrics = writable<SystemMetrics | null>(null);
export const alerts = writable<Alert[]>([]);
export const serverPort = writable<number | null>(null);
export const serverUptime = writable<number>(0);

// Derived stores
export const cpuPercent = derived(metrics, $m => $m?.cpu.overall_percent ?? 0);
export const memoryPercent = derived(metrics, $m => $m?.memory.used_percent ?? 0);
export const processes = derived(metrics, $m => $m?.top_processes ?? []);
export const loadAvg = derived(metrics, $m => $m?.load_avg ?? [0, 0, 0]);
export const coreCount = derived(metrics, $m => $m?.cpu.cores.length ?? 0);
export const systemInfo = derived(metrics, $m => $m?.system ?? null);
export const networkMetrics = derived(metrics, $m => $m?.network ?? null);
export const diskMetrics = derived(metrics, $m => $m?.disk ?? null);

// Critical/warning alerts count
export const criticalAlertsCount = derived(alerts, $a =>
  $a.filter(alert => alert.severity === 'critical').length
);
export const warningAlertsCount = derived(alerts, $a =>
  $a.filter(alert => alert.severity === 'warning').length
);
export const hasAlerts = derived(alerts, $a => $a.length > 0);

// WebSocket management
let ws: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
let pingInterval: ReturnType<typeof setInterval> | null = null;
let reconnectAttempts = 0;
const MAX_RECONNECT_ATTEMPTS = 10;
const RECONNECT_DELAY = 3000;
const PING_INTERVAL = 30000;

function getWebSocketUrl(): string {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const host = window.location.host;
  return `${protocol}//${host}/api/v1/ws`;
}

function startPing(): void {
  if (pingInterval) clearInterval(pingInterval);
  pingInterval = setInterval(() => {
    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({ type: 'ping' }));
    }
  }, PING_INTERVAL);
}

function stopPing(): void {
  if (pingInterval) {
    clearInterval(pingInterval);
    pingInterval = null;
  }
}

export function connect(): void {
  if (ws?.readyState === WebSocket.OPEN || ws?.readyState === WebSocket.CONNECTING) {
    return;
  }

  connecting.set(true);
  connectionError.set(null);

  try {
    const url = getWebSocketUrl();
    ws = new WebSocket(url);

    ws.onopen = () => {
      connected.set(true);
      connecting.set(false);
      connectionError.set(null);
      reconnectAttempts = 0;
      startPing();
    };

    ws.onmessage = (event: MessageEvent) => {
      try {
        const msg = JSON.parse(event.data) as WsMessage;

        switch (msg.type) {
          case 'metrics_update':
            if (msg.data && typeof msg.data === 'object') {
              metrics.set(msg.data as SystemMetrics);
              lastUpdate.set(new Date());

              // Extract alerts if present
              const data = msg.data as SystemMetrics;
              if (data.alerts) {
                alerts.set(data.alerts);
              }

              // Extract server info from timestamp
              if (data.system) {
                serverPort.set(data.system.port ?? null);
              }
            }
            break;

          case 'alert':
            // Handle new alert
            if (msg.data && typeof msg.data === 'object') {
              alerts.update(current => {
                const newAlert = msg.data as Alert;
                // Remove existing alert with same ID
                const filtered = current.filter(a => a.id !== newAlert.id);
                return [...filtered, newAlert];
              });
            }
            break;

          case 'alert_cleared':
            // Handle cleared alert
            if (msg.data && typeof msg.data === 'object') {
              const cleared = msg.data as { id: string };
              alerts.update(current => current.filter(a => a.id !== cleared.id));
            }
            break;

          case 'pong':
            // Connection is alive
            break;
        }
      } catch (err) {
        console.error('Failed to parse WebSocket message:', err);
      }
    };

    ws.onclose = () => {
      connected.set(false);
      connecting.set(false);
      stopPing();

      // Attempt to reconnect
      if (reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
        reconnectAttempts++;
        reconnectTimer = setTimeout(() => {
          connect();
        }, RECONNECT_DELAY * Math.min(reconnectAttempts, 3)); // Exponential backoff
      } else {
        connectionError.set('Connection lost. Please refresh the page.');
      }
    };

    ws.onerror = () => {
      connectionError.set('WebSocket error occurred');
      ws?.close();
    };
  } catch (err) {
    connecting.set(false);
    connectionError.set(err instanceof Error ? err.message : 'Failed to connect');
  }
}

export function disconnect(): void {
  if (reconnectTimer) {
    clearTimeout(reconnectTimer);
    reconnectTimer = null;
  }
  stopPing();
  ws?.close();
  ws = null;
  connected.set(false);
  connecting.set(false);
}

// Reconnect function for manual reconnection
export function reconnect(): void {
  disconnect();
  reconnectAttempts = 0;
  connect();
}

// API Base URL
const API_BASE = '/api/v1';

// API calls
export async function killProcess(pid: number): Promise<KillResponse> {
  const res = await fetch(`${API_BASE}/actions/kill`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ pid })
  });

  if (!res.ok) {
    const error = await res.text();
    throw new Error(error || `Failed to kill process ${pid}`);
  }

  return res.json() as Promise<KillResponse>;
}

export async function killStale(maxAgeHours: number = 24, dryRun: boolean = false): Promise<CleanupResponse> {
  const res = await fetch(`${API_BASE}/actions/kill-stale`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ max_age_hours: maxAgeHours, dry_run: dryRun })
  });

  if (!res.ok) {
    const error = await res.text();
    throw new Error(error || 'Failed to kill stale processes');
  }

  return res.json() as Promise<CleanupResponse>;
}

export async function cleanup(dryRun: boolean = false): Promise<CleanupResponse> {
  const res = await fetch(`${API_BASE}/actions/cleanup`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ include_stale: true, dry_run: dryRun })
  });

  if (!res.ok) {
    const error = await res.text();
    throw new Error(error || 'Failed to cleanup processes');
  }

  return res.json() as Promise<CleanupResponse>;
}

export async function fetchStaleProcesses(): Promise<StaleResponse> {
  const res = await fetch(`${API_BASE}/processes/stale`);

  if (!res.ok) {
    const error = await res.text();
    throw new Error(error || 'Failed to fetch stale processes');
  }

  return res.json() as Promise<StaleResponse>;
}

export async function fetchHealth(): Promise<HealthResponse> {
  const res = await fetch(`${API_BASE}/health`);

  if (!res.ok) {
    const error = await res.text();
    throw new Error(error || 'Failed to fetch health status');
  }

  const data = await res.json() as HealthResponse;
  serverUptime.set(data.uptime_seconds);
  serverPort.set(data.port);
  return data;
}

export async function fetchContainers(): Promise<ContainersResponse> {
  const res = await fetch(`${API_BASE}/containers`);

  if (!res.ok) {
    if (res.status === 404) {
      // Container endpoint not available
      return { containers: [], runtime: null };
    }
    const error = await res.text();
    throw new Error(error || 'Failed to fetch containers');
  }

  return res.json() as Promise<ContainersResponse>;
}

export async function restartContainer(containerId: string): Promise<{ success: boolean; message: string }> {
  const res = await fetch(`${API_BASE}/containers/${containerId}/restart`, {
    method: 'POST'
  });

  if (!res.ok) {
    const error = await res.text();
    throw new Error(error || `Failed to restart container ${containerId}`);
  }

  return res.json() as Promise<{ success: boolean; message: string }>;
}

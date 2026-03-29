export interface SystemMetrics {
  cpu: CpuMetrics;
  memory: MemoryMetrics;
  load_avg: [number, number, number];
  top_processes: ProcessInfo[];
  alerts: Alert[];
  system: SystemInfo;
  network: NetworkMetrics;
  disk: DiskMetrics;
  ports: PortInfo[];
  timestamp: string;
}

export interface PortInfo {
  port: number;
  protocol: string;
  address: string;
  pid: number;
  process_name: string;
  user: string;
  is_external: boolean;
  service: string | null;
}

export interface NetworkMetrics {
  interfaces: NetworkInterface[];
  total_received_bytes: number;
  total_transmitted_bytes: number;
}

export interface NetworkInterface {
  name: string;
  received_bytes: number;
  transmitted_bytes: number;
  received_packets: number;
  transmitted_packets: number;
}

export interface DiskMetrics {
  disks: DiskInfo[];
}

export interface DiskInfo {
  name: string;
  mount_point: string;
  file_system: string;
  total_bytes: number;
  available_bytes: number;
  used_bytes: number;
  used_percent: number;
  is_removable: boolean;
}

export interface SystemInfo {
  hostname: string;
  os_name: string;
  os_version: string;
  kernel_version: string;
  uptime_seconds: number;
  cpu_count: number;
  total_memory_bytes: number;
  port?: number;
  container_count?: number;
}

export interface CpuMetrics {
  overall_percent: number;
  cores: CoreMetrics[];
}

export interface CoreMetrics {
  id: number;
  usage_percent: number;
  frequency_mhz: number;
}

export interface MemoryMetrics {
  total_bytes: number;
  used_bytes: number;
  free_bytes: number;
  available_bytes: number;
  swap_total_bytes: number;
  swap_used_bytes: number;
  used_percent: number;
}

export interface ProcessInfo {
  pid: number;
  ppid: number | null;
  name: string;
  cpu_percent: number;
  memory_bytes: number;
  memory_percent: number;
  status: string;
  started_at: number;
  runtime_seconds: number;
  user: string;
  is_stale: boolean;
  is_zombie: boolean;
  command: string;
}

export interface StaleProcess {
  pid: number;
  name: string;
  cpu_percent: number;
  memory_bytes: number;
  runtime_hours: number;
  duplicate_count: number;
  stale_reason: string;
}

export interface Alert {
  id: string;
  alert_type: string;
  severity: 'info' | 'warning' | 'critical';
  message: string;
  value: number;
  threshold: number;
  since: string;
}

export interface Container {
  id: string;
  name: string;
  image: string;
  status: string;
  cpu_percent: number;
  memory_bytes: number;
  ports: string[];
}

export interface WsMessage {
  type: 'metrics_update' | 'alert' | 'alert_cleared' | 'pong';
  timestamp: string;
  sequence?: number;
  data: unknown;
}

export interface HealthResponse {
  status: string;
  uptime_seconds: number;
  version: string;
  pid: number;
  port: number;
}

export interface KillResponse {
  success: boolean;
  message: string;
}

export interface CleanupResponse {
  killed_zombies: number;
  killed_stale: number;
  total_killed: number;
  freed_bytes: number;
  processes: { pid: number; name: string; process_type: string }[];
}

export interface StaleResponse {
  processes: StaleProcess[];
  total_memory_waste_bytes: number;
  stale_count: number;
  zombie_count: number;
}

export interface ContainersResponse {
  containers: Container[];
  runtime: string | null;
}

export interface PortsResponse {
  ports: PortInfo[];
  total: number;
  external_count: number;
}

// Process management types
export type ProcessSignal = 'SIGTERM' | 'SIGHUP' | 'SIGINT' | 'SIGQUIT' | 'SIGSTOP' | 'SIGCONT' | 'SIGUSR1' | 'SIGUSR2';

export interface BatchKillRequest {
  pids: number[];
  signal?: string;
}

export interface BatchKillResult {
  pid: number;
  success: boolean;
  message: string;
}

export interface BatchKillResponse {
  results: BatchKillResult[];
  total_attempted: number;
  total_succeeded: number;
  total_failed: number;
}

export interface ColumnDef {
  key: string;
  label: string;
  sortable: boolean;
  defaultVisible: boolean;
}

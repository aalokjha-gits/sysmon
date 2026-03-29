import { describe, it, expect } from 'vitest';
import type {
  SystemMetrics,
  CpuMetrics,
  MemoryMetrics,
  ProcessInfo,
  Alert,
  DiskMetrics,
  NetworkMetrics,
  WsMessage,
  KillResponse,
  CleanupResponse,
  StaleResponse,
  HealthResponse,
  Container,
  ContainersResponse
} from './index';

function createMockMetrics(): SystemMetrics {
  return {
    cpu: {
      overall_percent: 45.2,
      cores: [
        { id: 0, usage_percent: 50.0, frequency_mhz: 3200 },
        { id: 1, usage_percent: 40.4, frequency_mhz: 3200 }
      ]
    },
    memory: {
      total_bytes: 17179869184,
      used_bytes: 8589934592,
      free_bytes: 4294967296,
      available_bytes: 8589934592,
      swap_total_bytes: 2147483648,
      swap_used_bytes: 0,
      used_percent: 50.0
    },
    load_avg: [2.5, 2.0, 1.5],
    top_processes: [
      {
        pid: 1234,
        ppid: 1,
        name: 'node',
        cpu_percent: 12.5,
        memory_bytes: 104857600,
        memory_percent: 0.6,
        status: 'running',
        started_at: 1700000000,
        runtime_seconds: 3600,
        user: '501',
        is_stale: false,
        is_zombie: false,
        command: 'node server.js'
      }
    ],
    alerts: [],
    system: {
      hostname: 'test-host',
      os_name: 'macOS',
      os_version: '14.0',
      kernel_version: '23.0.0',
      uptime_seconds: 86400,
      cpu_count: 8,
      total_memory_bytes: 17179869184
    },
    network: {
      interfaces: [
        {
          name: 'en0',
          received_bytes: 1000000,
          transmitted_bytes: 500000,
          received_packets: 1000,
          transmitted_packets: 500
        }
      ],
      total_received_bytes: 1000000,
      total_transmitted_bytes: 500000
    },
    disk: {
      disks: [
        {
          name: 'disk0s1',
          mount_point: '/',
          file_system: 'apfs',
          total_bytes: 500000000000,
          available_bytes: 250000000000,
          used_bytes: 250000000000,
          used_percent: 50.0,
          is_removable: false
        }
      ]
    },
    timestamp: '2024-01-01T00:00:00Z'
  };
}

describe('Type conformance', () => {
  it('SystemMetrics has all required fields', () => {
    const m = createMockMetrics();
    expect(m.cpu).toBeDefined();
    expect(m.memory).toBeDefined();
    expect(m.load_avg).toHaveLength(3);
    expect(m.top_processes).toBeInstanceOf(Array);
    expect(m.alerts).toBeInstanceOf(Array);
    expect(m.system).toBeDefined();
    expect(m.network).toBeDefined();
    expect(m.disk).toBeDefined();
    expect(m.timestamp).toBeDefined();
  });

  it('CpuMetrics structure is valid', () => {
    const cpu: CpuMetrics = { overall_percent: 55.0, cores: [] };
    expect(cpu.overall_percent).toBe(55.0);
    expect(cpu.cores).toEqual([]);
  });

  it('MemoryMetrics calculates used_percent correctly', () => {
    const mem: MemoryMetrics = {
      total_bytes: 16000000000,
      used_bytes: 8000000000,
      free_bytes: 4000000000,
      available_bytes: 8000000000,
      swap_total_bytes: 0,
      swap_used_bytes: 0,
      used_percent: 50.0
    };
    expect(mem.used_percent).toBe(50.0);
  });

  it('ProcessInfo has all fields including stale/zombie flags', () => {
    const proc: ProcessInfo = {
      pid: 42,
      ppid: 1,
      name: 'test',
      cpu_percent: 0.5,
      memory_bytes: 1024,
      memory_percent: 0.01,
      status: 'running',
      started_at: 0,
      runtime_seconds: 100,
      user: '501',
      is_stale: false,
      is_zombie: false,
      command: '/usr/bin/test'
    };
    expect(proc.pid).toBe(42);
    expect(proc.is_stale).toBe(false);
    expect(proc.is_zombie).toBe(false);
  });

  it('Alert severity is constrained', () => {
    const alert: Alert = {
      id: 'alert-1',
      alert_type: 'cpu_high',
      severity: 'critical',
      message: 'CPU usage high',
      value: 98.0,
      threshold: 95.0,
      since: '2024-01-01T00:00:00Z'
    };
    expect(['info', 'warning', 'critical']).toContain(alert.severity);
  });

  it('WsMessage type field is constrained', () => {
    const msg: WsMessage = {
      type: 'metrics_update',
      timestamp: '2024-01-01T00:00:00Z',
      sequence: 1,
      data: createMockMetrics()
    };
    expect(['metrics_update', 'alert', 'alert_cleared', 'pong']).toContain(msg.type);
  });
});

describe('JSON round-trip (simulates backend <-> frontend)', () => {
  it('SystemMetrics survives JSON serialization', () => {
    const original = createMockMetrics();
    const json = JSON.stringify(original);
    const parsed = JSON.parse(json) as SystemMetrics;

    expect(parsed.cpu.overall_percent).toBe(original.cpu.overall_percent);
    expect(parsed.memory.used_percent).toBe(original.memory.used_percent);
    expect(parsed.load_avg).toEqual(original.load_avg);
    expect(parsed.top_processes).toHaveLength(1);
    expect(parsed.top_processes[0].pid).toBe(1234);
    expect(parsed.system.hostname).toBe('test-host');
  });

  it('KillResponse round-trips', () => {
    const resp: KillResponse = { success: true, message: 'Killed process 42' };
    const parsed = JSON.parse(JSON.stringify(resp)) as KillResponse;
    expect(parsed.success).toBe(true);
    expect(parsed.message).toBe('Killed process 42');
  });

  it('CleanupResponse round-trips', () => {
    const resp: CleanupResponse = {
      killed_zombies: 2,
      killed_stale: 3,
      total_killed: 5,
      freed_bytes: 104857600,
      processes: [{ pid: 100, name: 'zombie1', process_type: 'zombie' }]
    };
    const parsed = JSON.parse(JSON.stringify(resp)) as CleanupResponse;
    expect(parsed.total_killed).toBe(5);
    expect(parsed.processes).toHaveLength(1);
  });

  it('StaleResponse round-trips', () => {
    const resp: StaleResponse = {
      processes: [
        {
          pid: 200,
          name: 'old-proc',
          cpu_percent: 0.1,
          memory_bytes: 50000,
          runtime_hours: 48.5,
          duplicate_count: 5,
          stale_reason: 'Runtime: 48.5h, CPU: 0.1%, Duplicates: 5'
        }
      ],
      total_memory_waste_bytes: 250000,
      stale_count: 1,
      zombie_count: 0
    };
    const parsed = JSON.parse(JSON.stringify(resp)) as StaleResponse;
    expect(parsed.stale_count).toBe(1);
    expect(parsed.processes[0].runtime_hours).toBe(48.5);
  });

  it('ContainersResponse round-trips', () => {
    const resp: ContainersResponse = {
      containers: [
        {
          id: 'abc123',
          name: 'web',
          image: 'nginx:latest',
          status: 'running',
          cpu_percent: 2.5,
          memory_bytes: 52428800,
          ports: ['80:80']
        }
      ],
      runtime: 'docker'
    };
    const parsed = JSON.parse(JSON.stringify(resp)) as ContainersResponse;
    expect(parsed.containers).toHaveLength(1);
    expect(parsed.runtime).toBe('docker');
  });
});

describe('Edge cases', () => {
  it('ProcessInfo ppid can be null', () => {
    const proc: ProcessInfo = {
      pid: 1,
      ppid: null,
      name: 'launchd',
      cpu_percent: 0,
      memory_bytes: 0,
      memory_percent: 0,
      status: 'running',
      started_at: 0,
      runtime_seconds: 999999,
      user: '0',
      is_stale: false,
      is_zombie: false,
      command: '/sbin/launchd'
    };
    expect(proc.ppid).toBeNull();
  });

  it('Empty metrics arrays are valid', () => {
    const cpu: CpuMetrics = { overall_percent: 0, cores: [] };
    const disk: DiskMetrics = { disks: [] };
    const net: NetworkMetrics = {
      interfaces: [],
      total_received_bytes: 0,
      total_transmitted_bytes: 0
    };
    expect(cpu.cores).toHaveLength(0);
    expect(disk.disks).toHaveLength(0);
    expect(net.interfaces).toHaveLength(0);
  });

  it('HealthResponse has expected fields', () => {
    const health: HealthResponse = {
      status: 'ok',
      uptime_seconds: 3600,
      version: '0.1.0',
      pid: 12345,
      port: 8080
    };
    expect(health.status).toBe('ok');
    expect(health.version).toBe('0.1.0');
  });
});

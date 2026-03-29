<script lang="ts">
  import { temperatureMetrics } from '$lib/stores/metrics';

  interface SensorDisplay {
    label: string;
    friendlyName: string;
    celsius: number | null;
    max: number | null;
    critical: number | null;
    status: 'cool' | 'normal' | 'warm' | 'hot' | 'critical' | 'unknown';
  }

  function friendlyLabel(raw: string): string {
    const lower = raw.toLowerCase();
    if (lower.includes('cpu') || lower.includes('tdie') || lower.includes('tctl'))
      return 'CPU';
    if (lower.includes('gpu'))
      return 'GPU';
    if (lower.includes('ssd') || lower.includes('nvme') || lower.includes('disk'))
      return 'Storage';
    if (lower.includes('battery') || lower.includes('batt'))
      return 'Battery';
    if (lower.includes('mem') || lower.includes('dram') || lower.includes('sodimm'))
      return 'Memory';
    if (lower.includes('pch'))
      return 'Chipset';
    if (lower.includes('fan'))
      return 'Fan';
    if (lower.includes('ambient') || lower.includes('airflow') || lower.includes('inlet'))
      return 'Ambient';
    if (lower.includes('pmu'))
      return 'Power';
    if (lower.includes('nand'))
      return 'NAND';
    if (lower.includes('wifi') || lower.includes('wlan'))
      return 'WiFi';
    if (lower.includes('thunderbolt') || lower.includes('tb'))
      return 'Thunderbolt';
    return raw.length > 20 ? raw.slice(0, 18) + '...' : raw;
  }

  function getStatus(celsius: number | null, critical: number | null): SensorDisplay['status'] {
    if (celsius === null) return 'unknown';
    if (critical !== null && celsius >= critical) return 'critical';
    if (celsius >= 85) return 'hot';
    if (celsius >= 65) return 'warm';
    if (celsius >= 40) return 'normal';
    return 'cool';
  }

  function getStatusLabel(status: SensorDisplay['status']): string {
    switch (status) {
      case 'cool': return 'Cool';
      case 'normal': return 'Normal';
      case 'warm': return 'Warm';
      case 'hot': return 'Hot';
      case 'critical': return 'Critical';
      default: return '--';
    }
  }

  function getStatusColor(status: SensorDisplay['status']): string {
    switch (status) {
      case 'cool': return 'var(--accent)';
      case 'normal': return 'var(--success)';
      case 'warm': return 'var(--warning)';
      case 'hot': return 'var(--danger)';
      case 'critical': return '#dc2626';
      default: return 'var(--text-muted)';
    }
  }

  let sensors = $derived.by((): SensorDisplay[] => {
    if (!$temperatureMetrics?.sensors) return [];

    const grouped = new Map<string, SensorDisplay>();

    for (const s of $temperatureMetrics.sensors) {
      if (s.temperature_celsius !== null && (s.temperature_celsius <= 0 || s.temperature_celsius >= 150)) continue;
      const friendly = friendlyLabel(s.label);
      const existing = grouped.get(friendly);
      const status = getStatus(s.temperature_celsius, s.critical_celsius);

      if (!existing || (s.temperature_celsius ?? 0) > (existing.celsius ?? 0)) {
        grouped.set(friendly, {
          label: s.label,
          friendlyName: friendly,
          celsius: s.temperature_celsius,
          max: s.max_celsius,
          critical: s.critical_celsius,
          status
        });
      }
    }

    return Array.from(grouped.values()).sort((a, b) => {
      const order: Record<string, number> = { CPU: 0, GPU: 1, Memory: 2, Storage: 3, Chipset: 4, Power: 5 };
      const oa = order[a.friendlyName] ?? 10;
      const ob = order[b.friendlyName] ?? 10;
      if (oa !== ob) return oa - ob;
      return (b.celsius ?? 0) - (a.celsius ?? 0);
    });
  });

  let avgTemp = $derived.by(() => {
    const temps = sensors.filter(s => s.celsius !== null).map(s => s.celsius!);
    if (temps.length === 0) return null;
    return temps.reduce((a, b) => a + b, 0) / temps.length;
  });

  let overallStatus = $derived.by(() => {
    if (sensors.length === 0) return 'unknown' as const;
    const statuses = sensors.map(s => s.status);
    if (statuses.includes('critical')) return 'critical' as const;
    if (statuses.includes('hot')) return 'hot' as const;
    if (statuses.includes('warm')) return 'warm' as const;
    if (statuses.includes('normal')) return 'normal' as const;
    return 'cool' as const;
  });
</script>

<div class="temp-compact compact-panel">
  <div class="panel-header">
    <span class="compact-panel-title">Temperature</span>
    <div class="overall-status">
      {#if avgTemp !== null}
        <span class="overall-badge" style:color={getStatusColor(overallStatus)}>
          {getStatusLabel(overallStatus)}
        </span>
        <span class="overall-avg mono" style:color={getStatusColor(overallStatus)}>
          {avgTemp.toFixed(0)}°C
        </span>
      {:else}
        <span class="overall-avg mono" style="color: var(--text-muted)">--</span>
      {/if}
    </div>
  </div>

  {#if sensors.length > 0}
    <div class="sensors-grid">
      {#each sensors as sensor}
        <div class="sensor-card" title="{sensor.label}: {sensor.celsius?.toFixed(1) ?? '--'}°C">
          <div class="sensor-top">
            <span class="sensor-name">{sensor.friendlyName}</span>
            <span class="sensor-status" style:color={getStatusColor(sensor.status)}>
              {getStatusLabel(sensor.status)}
            </span>
          </div>
          <div class="sensor-temp mono" style:color={getStatusColor(sensor.status)}>
            {sensor.celsius !== null ? `${sensor.celsius.toFixed(0)}°` : '--°'}
          </div>
          <div class="sensor-bar-bg">
            <div
              class="sensor-bar-fill"
              style:width="{sensor.celsius !== null ? Math.min(100, Math.max(0, (sensor.celsius / (sensor.critical ?? sensor.max ?? 105)) * 100)) : 0}%"
              style:background-color={getStatusColor(sensor.status)}
            ></div>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="empty-state">
      <span class="empty-text">No temperature sensors detected</span>
    </div>
  {/if}
</div>

<style>
  .compact-panel {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md, 6px);
    padding: 8px 10px;
    overflow: hidden;
  }

  .compact-panel-title {
    font-size: var(--font-xs);
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }

  .overall-status {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .overall-badge {
    font-size: var(--font-xs);
    font-weight: 600;
  }

  .overall-avg {
    font-size: var(--font-xs);
    font-weight: 700;
  }

  .sensors-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
    gap: 6px;
  }

  .sensor-card {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px 8px;
    background: var(--bg-elevated);
    border-radius: 4px;
  }

  .sensor-top {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 4px;
  }

  .sensor-name {
    font-size: var(--font-xs);
    color: var(--text-primary);
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .sensor-status {
    font-size: 10px;
    font-weight: 600;
    flex-shrink: 0;
  }

  .sensor-temp {
    font-size: var(--font-sm);
    font-weight: 700;
    line-height: 1.2;
  }

  .sensor-bar-bg {
    width: 100%;
    height: 3px;
    background: var(--bg-surface);
    border-radius: 2px;
    overflow: hidden;
    margin-top: 2px;
  }

  .sensor-bar-fill {
    height: 100%;
    border-radius: 2px;
    transition: width 300ms ease;
  }

  .empty-state {
    padding: 12px 0;
    text-align: center;
  }

  .empty-text {
    color: var(--text-muted);
    font-size: var(--font-xs);
  }

  .mono {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
</style>

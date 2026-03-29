<script lang="ts">
  import { formatBytes, formatPercent } from '$lib/utils/format';

  interface Props {
    label: string;
    value: number;
    unit?: string;
    color?: 'cpu' | 'memory' | 'load' | 'disk' | 'net' | 'default';
    icon?: string;
    extra?: number;
    extraIcon?: string;
  }

  let {
    label,
    value,
    unit = '',
    color = 'default',
    icon = '',
    extra = 0,
    extraIcon = ''
  }: Props = $props();

  function getColorClass(c: string): string {
    switch (c) {
      case 'cpu':
        return 'pill-cpu';
      case 'memory':
        return 'pill-memory';
      case 'load':
        return 'pill-load';
      case 'disk':
        return 'pill-disk';
      case 'net':
        return 'pill-net';
      default:
        return 'pill-default';
    }
  }

  function formatValue(): string {
    if (unit === '%') {
      return formatPercent(value);
    }
    if (label === 'NET') {
      return formatBytes(value);
    }
    if (label === 'LOAD') {
      return value.toFixed(1);
    }
    if (unit === '') {
      return value.toString();
    }
    return `${Number(value.toFixed(1))}${unit}`;
  }

  function formatExtra(): string {
    if (label === 'NET') {
      return formatBytes(extra);
    }
    return extra.toString();
  }
</script>

<div class="metric-pill {getColorClass(color)}">
  <span class="pill-label">{label}</span>
  <span class="pill-value">
    {#if icon}{icon}{/if}
    {formatValue()}
  </span>
  {#if extra > 0 && extraIcon}
    <span class="pill-extra">
      {extraIcon}{formatExtra()}
    </span>
  {/if}
</div>

<style>
  .metric-pill {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    border-radius: 4px;
    background: var(--bg-elevated);
    border: 1px solid var(--border-color);
    font-size: var(--font-sm);
    white-space: nowrap;
    transition: all var(--transition-fast);
  }

  .metric-pill:hover {
    background: var(--bg-surface-hover);
  }

  .pill-label {
    font-weight: 600;
    color: var(--text-secondary);
    font-size: var(--font-xs);
    text-transform: uppercase;
    letter-spacing: 0.025em;
  }

  .pill-value {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-weight: 600;
    color: var(--text-primary);
  }

  .pill-extra {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    font-size: var(--font-xs);
    color: var(--text-muted);
    margin-left: 4px;
    padding-left: 4px;
    border-left: 1px solid var(--border-color);
  }

  /* Color variants */
  .pill-cpu .pill-value {
    color: var(--cpu-color);
  }

  .pill-memory .pill-value {
    color: var(--memory-color);
  }

  .pill-load .pill-value {
    color: var(--success);
  }

  .pill-disk .pill-value {
    color: var(--info);
  }

  .pill-net .pill-value {
    color: var(--accent);
  }

  .pill-default .pill-value {
    color: var(--text-primary);
  }
</style>

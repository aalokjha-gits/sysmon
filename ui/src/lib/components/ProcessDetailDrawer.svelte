<script lang="ts">
  import { fade, fly } from 'svelte/transition';
  import { formatBytes, formatPercent, formatDuration } from '$lib/utils/format';
  import type { ProcessInfo } from '$lib/types';

  interface Props {
    process: ProcessInfo | null;
    isOpen: boolean;
    cpuHistory?: number[];
    onClose: () => void;
    onKill?: (process: ProcessInfo) => void;
  }

  let {
    process,
    isOpen,
    cpuHistory = [],
    onClose,
    onKill
  }: Props = $props();

  function handleKeydown(event: KeyboardEvent) {
    if (isOpen && event.key === 'Escape') {
      onClose();
    }
  }

  $effect(() => {
    if (isOpen) {
      document.addEventListener('keydown', handleKeydown);
    } else {
      document.removeEventListener('keydown', handleKeydown);
    }
    return () => {
      document.removeEventListener('keydown', handleKeydown);
    };
  });

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

  function getSparklinePath(history: number[]): string {
    if (!history || history.length === 0) return '';
    const width = 120;
    const height = 30;
    const maxVal = Math.max(100, ...history);
    
    return history.map((val, i) => {
      const x = (i / Math.max(1, history.length - 1)) * width;
      const y = height - (val / maxVal) * height;
      return `${i === 0 ? 'M' : 'L'} ${x.toFixed(1)} ${y.toFixed(1)}`;
    }).join(' ');
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }
</script>

{#if isOpen && process}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div 
    class="drawer-backdrop" 
    onclick={handleBackdropClick} 
    transition:fade={{ duration: 200 }}
  >
    <div 
      class="drawer-panel" 
      transition:fly={{ x: 500, duration: 300, opacity: 1 }}
    >
      <header class="drawer-header">
        <div class="header-main">
          <h2 class="process-name" title={process.name}>{process.name}</h2>
          <button class="close-btn" onclick={onClose} aria-label="Close drawer">×</button>
        </div>
        <div class="header-sub">
          <span class="pid">PID: {process.pid}</span>
          <span class="status-badge" style:color={getStatusColor(process.status)}>
            {getStatusIcon(process.status)} {process.status}
          </span>
        </div>
      </header>

      <div class="drawer-content">
        <div class="detail-section">
          <h3 class="section-title">Overview</h3>
          <div class="detail-grid">
            <div class="detail-item">
              <span class="detail-label">User</span>
              <span class="detail-value">{process.user || 'N/A'}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">PPID</span>
              <span class="detail-value">{process.ppid ?? 'N/A'}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">Runtime</span>
              <span class="detail-value">{formatDuration(process.runtime_seconds)}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">Started</span>
              <span class="detail-value">{new Date(process.started_at * 1000).toLocaleString()}</span>
            </div>
          </div>
        </div>

        <div class="detail-section">
          <h3 class="section-title">Resources</h3>
          
          <div class="resource-block">
            <div class="resource-header">
              <span class="resource-label">CPU</span>
              <span class="resource-value">{formatPercent(process.cpu_percent)}</span>
            </div>
            <div class="progress-bg">
              <div class="progress-bar cpu-bar" style:width="{Math.min(100, process.cpu_percent)}%"></div>
            </div>
            {#if cpuHistory && cpuHistory.length > 0}
              <div class="sparkline-wrapper">
                <svg class="sparkline" width="120" height="30" viewBox="0 0 120 30" preserveAspectRatio="none">
                  <path 
                    d={getSparklinePath(cpuHistory)} 
                    fill="none" 
                    stroke="var(--cpu-color)" 
                    stroke-width="2" 
                    vector-effect="non-scaling-stroke" 
                  />
                </svg>
              </div>
            {/if}
          </div>

          <div class="resource-block">
            <div class="resource-header">
              <span class="resource-label">Memory</span>
              <span class="resource-value">
                {formatBytes(process.memory_bytes)} <span class="mem-pct">({formatPercent(process.memory_percent)})</span>
              </span>
            </div>
            <div class="progress-bg">
              <div class="progress-bar mem-bar" style:width="{Math.min(100, process.memory_percent)}%"></div>
            </div>
          </div>
        </div>

        <div class="detail-section">
          <h3 class="section-title">Command Line</h3>
          <div class="cmd-box">
            <code>{process.command || process.name}</code>
          </div>
        </div>

        {#if process.is_stale || process.is_zombie}
          <div class="detail-section">
            <h3 class="section-title">Flags</h3>
            <div class="flags-container">
              {#if process.is_stale}
                <span class="badge stale-badge">⚠ Stale Process</span>
              {/if}
              {#if process.is_zombie}
                <span class="badge zombie-badge">☠ Zombie Process</span>
              {/if}
            </div>
          </div>
        {/if}
      </div>

      {#if onKill}
        <footer class="drawer-footer">
          <button class="btn-kill" onclick={() => onKill(process!)}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="3 6 5 6 21 6" />
              <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
            </svg>
            Kill Process
          </button>
        </footer>
      {/if}
    </div>
  </div>
{/if}

<style>
  .drawer-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(4px);
    z-index: 1000;
    display: flex;
    justify-content: flex-end;
  }

  .drawer-panel {
    width: 100%;
    max-width: 100%;
    height: 100vh;
    background: var(--bg-surface);
    border-left: 1px solid var(--border-color);
    box-shadow: -4px 0 24px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    color: var(--text-primary);
    overflow: hidden;
  }

  @media (min-width: 640px) {
    .drawer-panel {
      width: max(320px, 30vw);
    }
  }

  .drawer-header {
    padding: var(--space-4) var(--space-6);
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-elevated);
    flex-shrink: 0;
  }

  .header-main {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: var(--space-3);
  }

  .process-name {
    font-size: var(--font-xl);
    font-weight: 600;
    margin: 0;
    word-break: break-all;
    line-height: 1.2;
  }

  .close-btn {
    font-size: 1.5rem;
    line-height: 1;
    color: var(--text-secondary);
    padding: 0 var(--space-1);
    border-radius: var(--radius-sm);
    transition: all var(--transition-fast);
    background: transparent;
    border: none;
    cursor: pointer;
  }

  .close-btn:hover {
    color: var(--text-primary);
    background: var(--bg-surface-hover);
  }

  .header-sub {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-top: var(--space-2);
  }

  .pid {
    font-family: var(--font-mono);
    color: var(--text-muted);
    font-size: var(--font-sm);
  }

  .status-badge {
    font-size: var(--font-xs);
    font-weight: 500;
  }

  .drawer-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-6);
    display: flex;
    flex-direction: column;
    gap: var(--space-8);
  }

  .detail-section {
    display: flex;
    flex-direction: column;
  }

  .section-title {
    font-size: var(--font-sm);
    color: var(--text-muted);
    margin: 0 0 var(--space-4) 0;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .detail-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-4);
  }

  .detail-item {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .detail-label {
    font-size: var(--font-xs);
    color: var(--text-muted);
  }

  .detail-value {
    font-size: var(--font-sm);
    font-weight: 500;
  }

  .resource-block {
    margin-bottom: var(--space-4);
  }

  .resource-block:last-child {
    margin-bottom: 0;
  }

  .resource-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-2);
    font-size: var(--font-sm);
  }

  .resource-label {
    color: var(--text-secondary);
    font-weight: 500;
  }

  .resource-value {
    font-family: var(--font-mono);
    font-weight: 500;
  }
  
  .mem-pct {
    color: var(--text-muted);
    font-size: var(--font-xs);
  }

  .progress-bg {
    background: var(--bg-surface-hover);
    height: 6px;
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .progress-bar {
    height: 100%;
    border-radius: var(--radius-sm);
    transition: width var(--transition-normal);
  }

  .cpu-bar {
    background: var(--cpu-color);
  }

  .mem-bar {
    background: var(--memory-color);
  }

  .sparkline-wrapper {
    margin-top: var(--space-3);
    height: 30px;
    width: 120px;
  }

  .sparkline {
    width: 100%;
    height: 100%;
  }

  .cmd-box {
    background: var(--bg-elevated);
    padding: var(--space-3);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-color);
    overflow-x: auto;
  }

  .cmd-box code {
    font-family: var(--font-mono);
    font-size: var(--font-xs);
    color: var(--text-primary);
    white-space: pre-wrap;
    word-break: break-all;
  }

  .flags-container {
    display: flex;
    gap: var(--space-2);
    flex-wrap: wrap;
  }

  .badge {
    font-size: var(--font-xs);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-weight: 500;
  }

  .stale-badge {
    background: rgba(245, 158, 11, 0.15);
    color: var(--warning);
    border: 1px solid rgba(245, 158, 11, 0.3);
  }

  .zombie-badge {
    background: rgba(239, 68, 68, 0.15);
    color: var(--danger);
    border: 1px solid rgba(239, 68, 68, 0.3);
  }

  .drawer-footer {
    padding: var(--space-4) var(--space-6);
    border-top: 1px solid var(--border-color);
    background: var(--bg-elevated);
    flex-shrink: 0;
  }

  .btn-kill {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    background: var(--danger);
    color: white;
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-md);
    font-size: var(--font-sm);
    font-weight: 500;
    transition: all var(--transition-fast);
    border: none;
    cursor: pointer;
  }

  .btn-kill:hover {
    background: var(--danger-hover);
  }

  .btn-kill svg {
    width: 16px;
    height: 16px;
  }
</style>

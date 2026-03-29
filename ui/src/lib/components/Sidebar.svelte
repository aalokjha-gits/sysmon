<script lang="ts">
  import { activeView, setView, views, type ViewId } from '$lib/stores/navigation';
  import { criticalAlertsCount, warningAlertsCount } from '$lib/stores/metrics';

  interface IconMap {
    [key: string]: { path: string; viewBox?: string };
  }

  const icons: IconMap = {
    overview: {
      path: 'M3 3h7v7H3V3zm11 0h7v7h-7V3zm0 11h7v7h-7v-7zM3 14h7v7H3v-7z',
    },
    processes: {
      path: 'M4 6h16M4 12h16M4 18h10',
    },
    network: {
      path: 'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm-1-14v4.17l-2.59-2.58L7 9l5 5 5-5-1.41-1.41L13 10.17V6h-2z',
      viewBox: '0 0 24 24',
    },
    containers: {
      path: 'M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z',
    },
    alerts: {
      path: 'M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0zM12 9v4M12 17h.01',
    },
  };

  function getIcon(viewId: string): { path: string; viewBox: string } {
    const icon = icons[viewId];
    return {
      path: icon?.path ?? '',
      viewBox: icon?.viewBox ?? '0 0 24 24',
    };
  }

  function getBadge(viewId: string): number {
    if (viewId === 'alerts') {
      return $criticalAlertsCount + $warningAlertsCount;
    }
    return 0;
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
    const num = parseInt(e.key);
    if (num >= 1 && num <= views.length) {
      setView(views[num - 1].id);
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<nav class="sidebar" aria-label="Main navigation">
  {#each views as view (view.id)}
    {@const icon = getIcon(view.id)}
    {@const badge = getBadge(view.id)}
    {@const isActive = $activeView === view.id}
    <button
      class="nav-item"
      class:active={isActive}
      onclick={() => setView(view.id)}
      title="{view.label} ({view.shortcut})"
      aria-current={isActive ? 'page' : undefined}
    >
      <div class="icon-wrapper">
        <svg viewBox={icon.viewBox} fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d={icon.path} />
        </svg>
        {#if badge > 0}
          <span class="badge" class:critical={$criticalAlertsCount > 0}>{badge}</span>
        {/if}
      </div>
      <span class="nav-label">{view.label}</span>
    </button>
  {/each}
</nav>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    width: 148px;
    background: var(--bg-surface);
    border-right: 1px solid var(--border-color);
    padding: 6px 0;
    gap: 2px;
    flex-shrink: 0;
    overflow: hidden;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    margin: 0 6px;
    border-radius: var(--radius-md);
    color: var(--text-muted);
    background: transparent;
    border: none;
    cursor: pointer;
    position: relative;
    transition: all var(--transition-fast);
    justify-content: flex-start;
    min-height: 36px;
  }

  .nav-item:hover {
    color: var(--text-primary);
    background: var(--bg-surface-hover);
  }

  .nav-item.active {
    color: var(--accent);
    background: rgba(59, 130, 246, 0.1);
  }

  .nav-item.active::before {
    content: '';
    position: absolute;
    left: -6px;
    top: 6px;
    bottom: 6px;
    width: 3px;
    background: var(--accent);
    border-radius: 0 2px 2px 0;
  }

  .icon-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .icon-wrapper svg {
    width: 18px;
    height: 18px;
  }

  .badge {
    position: absolute;
    top: -6px;
    right: -8px;
    min-width: 14px;
    height: 14px;
    padding: 0 3px;
    border-radius: 7px;
    background: var(--warning);
    color: #000;
    font-size: 9px;
    font-weight: 700;
    display: flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
  }

  .badge.critical {
    background: var(--danger);
    color: #fff;
  }

  .nav-label {
    font-size: var(--font-xs);
    font-weight: 500;
    white-space: nowrap;
  }

  /* Mobile: bottom nav */
  @media (max-width: 768px) {
    .sidebar {
      flex-direction: row;
      width: 100% !important;
      height: 48px;
      border-right: none;
      border-top: 1px solid var(--border-color);
      padding: 0 4px;
      gap: 0;
      justify-content: space-around;
      order: 99;
    }

    .nav-item {
      flex-direction: column;
      gap: 2px;
      padding: 4px 6px;
      margin: 0;
      min-height: unset;
      border-radius: var(--radius-sm);
    }

    .nav-item.active::before {
      left: 4px;
      right: 4px;
      top: -1px;
      bottom: unset;
      width: unset;
      height: 2px;
      border-radius: 0 0 2px 2px;
    }

    .icon-wrapper svg {
      width: 16px;
      height: 16px;
    }

    .nav-label {
      display: block;
      font-size: 9px;
    }

    .shortcut {
      display: none;
    }
  }
</style>

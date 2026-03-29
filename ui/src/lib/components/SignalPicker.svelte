<script lang="ts">
  import type { ProcessSignal } from '$lib/types';
  import { onMount } from 'svelte';

  interface Props {
    selected?: string;
    onSelect: (signal: string) => void;
    disabled?: boolean;
    compact?: boolean;
  }

  let {
    selected = 'SIGTERM',
    onSelect,
    disabled = false,
    compact = false
  }: Props = $props();

  let isOpen = $state(false);
  let container: HTMLDivElement | undefined = $state();

  const signals: { name: ProcessSignal; num: number; desc: string }[] = [
    { name: 'SIGTERM', num: 15, desc: 'Graceful termination' },
    { name: 'SIGHUP', num: 1, desc: 'Reload configuration' },
    { name: 'SIGINT', num: 2, desc: 'Interrupt' },
    { name: 'SIGQUIT', num: 3, desc: 'Quit with core dump' },
    { name: 'SIGSTOP', num: 17, desc: 'Pause process' },
    { name: 'SIGCONT', num: 19, desc: 'Resume process' },
    { name: 'SIGUSR1', num: 10, desc: 'User signal 1' },
    { name: 'SIGUSR2', num: 12, desc: 'User signal 2' }
  ];

  let currentSignal = $derived(signals.find(s => s.name === selected) || signals[0]);

  function toggle() {
    if (!disabled) {
      isOpen = !isOpen;
    }
  }

  function selectSignal(signalName: string) {
    onSelect(signalName);
    isOpen = false;
  }

  function handleOutsideClick(event: MouseEvent) {
    if (isOpen && container && !container.contains(event.target as Node)) {
      isOpen = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (isOpen && event.key === 'Escape') {
      isOpen = false;
    }
  }

  $effect(() => {
    if (isOpen) {
      window.addEventListener('click', handleOutsideClick);
      window.addEventListener('keydown', handleKeydown);
      
      return () => {
        window.removeEventListener('click', handleOutsideClick);
        window.removeEventListener('keydown', handleKeydown);
      };
    }
  });
</script>

<div class="signal-picker" bind:this={container} class:is-compact={compact}>
  <button 
    type="button"
    class="trigger"
    {disabled}
    onclick={toggle}
    aria-haspopup="listbox"
    aria-expanded={isOpen}
  >
    <div class="trigger-content">
      <span class="signal-name">{currentSignal.name}</span>
      {#if !compact}
        <span class="signal-desc">({currentSignal.num}) &mdash; {currentSignal.desc}</span>
      {/if}
    </div>
    <svg class="chevron" class:open={isOpen} width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <polyline points="6 9 12 15 18 9"></polyline>
    </svg>
  </button>

  {#if isOpen}
    <div class="dropdown animate-slide-down" role="listbox">
      {#each signals as signal}
        <button
          type="button"
          class="option"
          class:selected={selected === signal.name}
          role="option"
          aria-selected={selected === signal.name}
          onclick={() => selectSignal(signal.name)}
        >
          <div class="option-left">
            <span class="option-name">{signal.name}</span>
            <span class="option-num">{signal.num}</span>
          </div>
          <span class="option-desc">{signal.desc}</span>
          {#if selected === signal.name}
            <svg class="check" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="20 6 9 17 4 12"></polyline>
            </svg>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .signal-picker {
    position: relative;
    display: inline-block;
    width: 100%;
    max-width: 320px;
    font-family: var(--font-mono, monospace);
  }

  .signal-picker.is-compact {
    max-width: max-content;
  }

  .trigger {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: var(--space-2) var(--space-3);
    background-color: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    transition: all var(--transition-fast);
    min-height: 2.25rem;
  }

  .trigger:hover:not(:disabled) {
    background-color: var(--bg-surface-hover);
    border-color: var(--text-muted);
  }

  .trigger:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .trigger-content {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    text-align: left;
    overflow: hidden;
  }

  .signal-name {
    font-weight: 600;
    font-size: var(--font-sm);
    color: var(--text-primary);
  }

  .signal-desc {
    font-size: var(--font-xs);
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .chevron {
    flex-shrink: 0;
    margin-left: var(--space-2);
    color: var(--text-muted);
    transition: transform var(--transition-fast);
  }

  .chevron.open {
    transform: rotate(180deg);
  }

  .dropdown {
    position: absolute;
    top: calc(100% + var(--space-1));
    left: 0;
    z-index: 50;
    width: max-content;
    min-width: 100%;
    background-color: var(--bg-elevated);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
    padding: var(--space-1);
    max-height: 300px;
    overflow-y: auto;
  }

  .option {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: var(--space-4);
    width: 100%;
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    text-align: left;
    transition: background-color var(--transition-fast);
  }

  .option:hover {
    background-color: var(--bg-surface-hover);
  }

  .option.selected {
    background-color: rgba(59, 130, 246, 0.15); /* var(--accent) with opacity */
  }

  .option-left {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .option-name {
    font-weight: 600;
    font-size: var(--font-sm);
    color: var(--text-primary);
  }

  .option.selected .option-name {
    color: var(--accent);
  }

  .option-num {
    font-size: var(--font-xs);
    color: var(--text-muted);
    background-color: var(--bg-surface);
    padding: 0 var(--space-1);
    border-radius: var(--radius-sm);
    min-width: 1.5rem;
    text-align: center;
  }

  .option-desc {
    font-size: var(--font-sm);
    color: var(--text-secondary);
  }

  .check {
    color: var(--accent);
    flex-shrink: 0;
  }
</style>

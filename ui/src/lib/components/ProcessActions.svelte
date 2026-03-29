<script lang="ts">
  import { fly, fade } from 'svelte/transition';

  interface Props {
    selectedCount: number;
    onKill: () => void;
    onSignal: (signal: string) => void;
    onClear: () => void;
  }

  let {
    selectedCount,
    onKill,
    onSignal,
    onClear
  }: Props = $props();

  let showSignalDropdown = $state(false);
  const signals = ['SIGTERM', 'SIGHUP', 'SIGSTOP', 'SIGCONT', 'SIGINT'];

  function handleSignal(signal: string) {
    onSignal(signal);
    showSignalDropdown = false;
  }

  function handleOutsideClick() {
    if (showSignalDropdown) {
      showSignalDropdown = false;
    }
  }
</script>

<svelte:window onclick={handleOutsideClick} />

{#if selectedCount > 0}
  <div class="actions-wrapper" transition:fly={{ y: 20, duration: 250 }}>
    <div class="actions-bar">
      <span class="count-text">{selectedCount} {selectedCount === 1 ? 'process' : 'processes'} selected</span>
      
      <div class="buttons">
        <button class="btn btn-clear" onclick={onClear}>
          Clear
        </button>
        
        <div class="dropdown-container">
          <button 
            class="btn btn-signal" 
            onclick={(e) => {
              e.stopPropagation();
              showSignalDropdown = !showSignalDropdown;
            }}
          >
            Send Signal
          </button>
          
          {#if showSignalDropdown}
            <div 
              class="dropdown-menu"
              transition:fade={{ duration: 150 }}
            >
              {#each signals as signal}
                <button 
                  class="dropdown-item" 
                  onclick={() => handleSignal(signal)}
                >
                  {signal}
                </button>
              {/each}
            </div>
          {/if}
        </div>
        
        <button class="btn btn-danger" onclick={onKill}>
          Kill Selected
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .actions-wrapper {
    position: sticky;
    bottom: 24px;
    left: 0;
    right: 0;
    display: flex;
    justify-content: center;
    z-index: 20;
    pointer-events: none;
    margin-top: 16px;
  }

  .actions-bar {
    pointer-events: auto;
    background: var(--bg-elevated);
    border: 1px solid var(--border-color);
    box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.5), 0 8px 10px -6px rgba(0, 0, 0, 0.5);
    border-radius: 9999px;
    padding: 8px 16px;
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .count-text {
    font-size: var(--font-sm);
    color: var(--text-primary);
    font-weight: 500;
  }

  .buttons {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .btn {
    padding: 4px 12px;
    border-radius: var(--radius-md);
    font-size: var(--font-sm);
    font-weight: 500;
    transition: all var(--transition-fast);
    border: 1px solid transparent;
    cursor: pointer;
  }

  .btn-danger {
    background: var(--danger);
    color: white;
  }

  .btn-danger:hover {
    background: var(--danger-hover);
  }

  .btn-signal {
    background: var(--bg-surface);
    border-color: var(--border-color);
    color: var(--text-primary);
  }

  .btn-signal:hover {
    background: var(--bg-surface-hover);
  }

  .btn-clear {
    background: transparent;
    color: var(--text-muted);
  }

  .btn-clear:hover {
    color: var(--text-primary);
    background: var(--bg-surface);
  }

  .dropdown-container {
    position: relative;
    display: flex;
  }

  .dropdown-menu {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 50%;
    transform: translateX(-50%);
    background: var(--bg-elevated);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    padding: 4px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.5);
    min-width: 120px;
    z-index: 30;
  }

  .dropdown-item {
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    font-size: var(--font-xs);
    color: var(--text-primary);
    text-align: left;
    transition: background var(--transition-fast);
    background: transparent;
    border: none;
    cursor: pointer;
    width: 100%;
  }

  .dropdown-item:hover {
    background: var(--bg-surface-hover);
  }
</style>

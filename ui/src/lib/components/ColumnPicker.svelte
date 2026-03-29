<script lang="ts">
  interface Props {
    columns: { key: string; label: string }[];
    visible: Set<string>;
    onChange: (key: string) => void;
  }

  let { columns, visible, onChange }: Props = $props();

  let isOpen = $state(false);
  let dropdownRef = $state<HTMLElement | null>(null);
  let buttonRef = $state<HTMLButtonElement | null>(null);

  function toggle(event: MouseEvent) {
    event.stopPropagation();
    isOpen = !isOpen;
  }

  function handleClickOutside(event: MouseEvent) {
    if (
      isOpen &&
      dropdownRef &&
      !dropdownRef.contains(event.target as Node) &&
      buttonRef &&
      !buttonRef.contains(event.target as Node)
    ) {
      isOpen = false;
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Escape' && isOpen) {
      isOpen = false;
    }
  }

  function handleChange(key: string) {
    if (visible.has(key) && visible.size <= 2) {
      return;
    }
    onChange(key);
  }
</script>

<svelte:window onclick={handleClickOutside} onkeydown={handleKeyDown} />

<div class="column-picker">
  <button
    class="trigger"
    bind:this={buttonRef}
    onclick={toggle}
    aria-label="Toggle columns"
    aria-expanded={isOpen}
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <circle cx="12" cy="12" r="3"></circle>
      <path
        d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"
      ></path>
    </svg>
  </button>

  {#if isOpen}
    <div class="dropdown animate-slide-down" bind:this={dropdownRef}>
      <div class="dropdown-header">Columns</div>
      <div class="dropdown-list">
        {#each columns as { key, label }}
          {@const isChecked = visible.has(key)}
          {@const isDisabled = isChecked && visible.size <= 2}
          <label class="item" class:disabled={isDisabled}>
            <div class="checkbox" class:checked={isChecked}>
              <input
                type="checkbox"
                checked={isChecked}
                disabled={isDisabled}
                onchange={() => handleChange(key)}
                class="visually-hidden"
              />
              {#if isChecked}
                <svg
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="3"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <polyline points="20 6 9 17 4 12"></polyline>
                </svg>
              {/if}
            </div>
            <span class="label">{label}</span>
          </label>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .column-picker {
    position: relative;
    display: inline-block;
  }

  .trigger {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 3px 8px;
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    transition: all var(--transition-fast);
  }

  .trigger:hover {
    color: var(--text-primary);
    background-color: var(--bg-surface-hover);
  }

  .trigger[aria-expanded='true'] {
    color: var(--accent);
    background-color: var(--bg-surface-hover);
  }

  .dropdown {
    position: absolute;
    top: calc(100% + var(--space-1));
    right: 0;
    z-index: 50;
    min-width: 180px;
    background-color: var(--bg-elevated);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
    padding: var(--space-1) 0;
    font-size: var(--font-xs);
  }

  .dropdown-header {
    padding: var(--space-1) var(--space-3);
    color: var(--text-muted);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-size: 0.7rem;
    margin-bottom: var(--space-1);
  }

  .dropdown-list {
    display: flex;
    flex-direction: column;
  }

  .item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-compact) var(--space-3);
    cursor: pointer;
    color: var(--text-primary);
    transition: background-color var(--transition-fast);
  }

  .item:hover:not(.disabled) {
    background-color: var(--bg-surface-hover);
  }

  .item.disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .checkbox {
    width: 14px;
    height: 14px;
    border: 1px solid var(--text-muted);
    border-radius: 3px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition-fast);
    background-color: transparent;
    flex-shrink: 0;
  }

  .checkbox.checked {
    background-color: var(--accent);
    border-color: var(--accent);
    color: white;
  }

  .checkbox svg {
    width: 10px;
    height: 10px;
  }

  .label {
    user-select: none;
    white-space: nowrap;
  }
</style>

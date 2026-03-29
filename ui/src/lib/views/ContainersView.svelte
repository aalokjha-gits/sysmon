<script lang="ts">
  import ContainersCompact from '$lib/components/ContainersCompact.svelte';
  import StaleCompact from '$lib/components/StaleCompact.svelte';
</script>

<div class="containers-view">
  <div class="section">
    <h2 class="section-title">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="section-icon">
        <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
      </svg>
      Containers
    </h2>
    <div class="section-content">
      <ContainersCompact />
    </div>
  </div>

  <div class="section">
    <h2 class="section-title">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="section-icon">
        <circle cx="12" cy="12" r="10" />
        <path d="M12 6v6l4 2" />
      </svg>
      Stale Processes
    </h2>
    <div class="section-content">
      <StaleCompact />
    </div>
  </div>
</div>

<style>
  .containers-view {
    height: 100%;
    overflow-y: auto;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .section {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .section-title {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    font-size: var(--font-sm);
    font-weight: 600;
    color: var(--text-primary);
    border-bottom: 1px solid var(--border-color);
    margin: 0;
  }

  .section-icon {
    width: 16px;
    height: 16px;
    color: var(--accent);
    flex-shrink: 0;
  }

  .section-content {
    padding: 0;
  }
</style>

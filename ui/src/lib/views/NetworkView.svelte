<script lang="ts">
  import NetworkCompact from '$lib/components/NetworkCompact.svelte';
  import PortsCompact from '$lib/components/PortsCompact.svelte';
</script>

<div class="network-view">
  <div class="section">
    <h2 class="section-title">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="section-icon">
        <path d="M5 12h14M12 5l7 7-7 7" />
      </svg>
      Network Interfaces
    </h2>
    <div class="section-content">
      <NetworkCompact />
    </div>
  </div>

  <div class="section">
    <h2 class="section-title">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="section-icon">
        <circle cx="12" cy="12" r="10" />
        <line x1="2" y1="12" x2="22" y2="12" />
      </svg>
      Listening Ports
    </h2>
    <div class="section-content full-width">
      <PortsCompact />
    </div>
  </div>
</div>

<style>
  .network-view {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    padding: 12px;
    align-items: stretch;
    height: 100%;
  }

  @media (max-width: 1024px) {
    .network-view {
      grid-template-columns: 1fr;
    }
  }

  .section {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    overflow-y: auto;
    max-height: calc(100vh - 160px);
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

  .section-content.full-width :global(> *) {
    max-height: none;
  }

  .section-content :global(.ports-list) {
    max-height: none;
  }
</style>
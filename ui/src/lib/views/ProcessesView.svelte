<script lang="ts">
  import ProcessTable from '$lib/components/ProcessTable.svelte';
</script>

<div class="processes-view">
  <ProcessTable />
</div>

<style>
  .processes-view {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
</style>

<script lang="ts">
  let {
    resultCount = 0,
    searchTime = null,
  }: {
    resultCount: number;
    searchTime: number | null;
  } = $props();
</script>

<div class="status-bar">
  <div class="status-left">
    <span class="status-badge">Vanta</span>
  </div>
  <div class="status-right">
    {#if searchTime !== null}
      <span class="status-info">
        {resultCount} result{resultCount !== 1 ? "s" : ""}
        · {searchTime.toFixed(1)}ms
      </span>
    {/if}
  </div>
</div>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    border-top: 1px solid var(--vanta-border);
    font-size: 11px;
  }

  .status-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-badge {
    background: var(--vanta-accent);
    color: white;
    padding: 1px 8px;
    border-radius: 4px;
    font-weight: 600;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .status-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-info {
    color: var(--vanta-text-dim);
    opacity: 0.6;
  }
</style>

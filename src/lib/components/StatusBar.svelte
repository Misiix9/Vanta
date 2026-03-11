<script lang="ts">
  import { onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";

  let {
    resultCount = 0,
    searchTime = null,
  }: {
    resultCount: number;
    searchTime: number | null;
  } = $props();

  let appVersion = $state("…");

  onMount(async () => {
    appVersion = await getVersion();
  });
</script>

<div class="status-bar v2-hstack">
  <div class="status-left">
    <span class="status-badge v2-status-badge info">
      <img src="/32x32.png" alt="" class="status-badge-logo" />
      Vanta
    </span>
  </div>

  <div class="status-right">
    {#if searchTime !== null}
      <span class="status-info">
        {resultCount} result{resultCount !== 1 ? "s" : ""}
        · {searchTime.toFixed(1)}ms
      </span>
    {/if}
    <span class="status-version">v{appVersion}</span>
  </div>
</div>

<style>
  .status-bar {
    justify-content: space-between;
    padding: calc(var(--space-1) * var(--vanta-space-scale, 1)) calc(var(--space-3) * var(--vanta-space-scale, 1));
    font-size: var(--type-caption);
    color: var(--ds-text-secondary, #999);
    border-top: 1px solid var(--ds-border, rgba(255, 255, 255, 0.08));
    flex-shrink: 0;
    user-select: none;
  }

  .status-left,
  .status-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-badge {
    gap: 4px;
    font-weight: var(--weight-medium);
  }

  .status-badge-logo {
    width: 12px;
    height: 12px;
    opacity: 0.6;
  }

  .status-info {
    color: var(--ds-text-secondary, #999);
  }

  .status-version {
    opacity: 0.5;
  }
</style>

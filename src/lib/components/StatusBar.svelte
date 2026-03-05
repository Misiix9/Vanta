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

<div class="status-bar">
  <div class="status-left">
    <span class="status-badge">
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
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 16px;
    font-size: 11px;
    color: var(--vanta-text-dim, #666);
    border-top: 1px solid var(--vanta-border, rgba(255, 255, 255, 0.06));
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
    display: flex;
    align-items: center;
    gap: 4px;
    font-weight: 500;
    color: var(--vanta-text-dim, #888);
  }

  .status-badge-logo {
    width: 12px;
    height: 12px;
    opacity: 0.6;
  }

  .status-info {
    color: var(--vanta-text-dim, #888);
  }

  .status-version {
    opacity: 0.5;
  }
</style>

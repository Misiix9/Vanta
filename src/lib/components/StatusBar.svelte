<script lang="ts">
  import { onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";

  let {
    resultCount = 0,
    searchTime = null,
    isSearching = false,
    notificationCount = 0,
    onOpenNotifications,
  }: {
    resultCount: number;
    searchTime: number | null;
    isSearching?: boolean;
    notificationCount?: number;
    onOpenNotifications?: () => void;
  } = $props();

  let appVersion = $state("…");

  onMount(async () => {
    appVersion = await getVersion();
  });
</script>

<div class="status-bar v2-hstack" role="status" aria-live="polite" aria-atomic="true">
  <div class="status-left">
    <span class="status-badge v2-status-badge info">
      <img src="/32x32.png" alt="" class="status-badge-logo" />
      Vanta
    </span>
  </div>

  <div class="status-right">
    {#if isSearching}
      <span class="status-info status-searching" aria-label="Searching">
        <span class="search-spinner" aria-hidden="true"></span>
        Searching…
      </span>
    {:else if searchTime !== null}
      <span class="status-info" aria-label="{resultCount} result{resultCount !== 1 ? 's' : ''} found">
        {resultCount} result{resultCount !== 1 ? "s" : ""}
        · {searchTime.toFixed(1)}ms
      </span>
    {/if}
    <span class="status-version">v{appVersion}</span>
    <button
      class="status-notifications"
      onclick={() => onOpenNotifications?.()}
      aria-label="Open notifications"
      title="Open notifications"
    >
      <span>Notifications</span>
      {#if notificationCount > 0}
        <span class="notification-count">{notificationCount}</span>
      {/if}
    </button>
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
    width: var(--icon-xs);
    height: var(--icon-xs);
    opacity: 0.6;
  }

  .status-info {
    color: var(--ds-text-secondary, #999);
  }

  .status-version {
    opacity: 0.5;
  }

  .status-notifications {
    border: 1px solid var(--ds-border, rgba(255, 255, 255, 0.12));
    border-radius: 999px;
    padding: 2px 8px;
    font-size: 11px;
    color: var(--ds-text-secondary, #999);
    background: rgba(255, 255, 255, 0.04);
    display: inline-flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
  }

  .status-notifications:hover {
    color: var(--ds-text-primary, #ddd);
  }

  .notification-count {
    min-width: 16px;
    height: 16px;
    border-radius: 999px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 10px;
    background: rgba(96, 165, 250, 0.25);
    color: #dbeafe;
    padding: 0 4px;
  }

  .status-searching {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .search-spinner {
    display: inline-block;
    width: 10px;
    height: 10px;
    border: 1.5px solid var(--ds-text-secondary, #999);
    border-top-color: var(--ds-accent, #6af);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>

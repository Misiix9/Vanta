<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";

  let {
    resultCount = 0,
    searchTime = null,
  }: {
    resultCount: number;
    searchTime: number | null;
  } = $props();

  let downloadStatus = $state<"idle" | "downloading" | "success" | "failed">(
    "idle",
  );
  let unlisten: () => void;

  onMount(async () => {
    unlisten = await listen<{ status: string }>("download_status", (event) => {
      downloadStatus = event.payload.status as any;
      if (downloadStatus === "success" || downloadStatus === "failed") {
        setTimeout(() => {
          downloadStatus = "idle";
        }, 3000);
      }
    });
  });

  onDestroy(() => {
    if (unlisten) unlisten();
  });
</script>

<div class="status-bar">
  <div class="status-left">
    <span class="status-badge">Vanta</span>
  </div>
  <div class="status-right">
    {#if downloadStatus !== "idle"}
      <div
        class="status-download"
        class:success={downloadStatus === "success"}
        class:failed={downloadStatus === "failed"}
      >
        {#if downloadStatus === "downloading"}
          <span class="spinner"></span>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="12"
            height="12"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            style="opacity: 0.8"
          >
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="7 10 12 15 17 10" />
            <line x1="12" x2="12" y1="15" y2="3" />
          </svg>
          Downloading
        {:else if downloadStatus === "success"}
          ✓ Downloaded
        {:else if downloadStatus === "failed"}
          ✗ Failed
        {/if}
      </div>
    {/if}
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

  .status-download {
    display: flex;
    align-items: center;
    gap: 6px;
    font-weight: 500;
    margin-right: 8px;
  }

  .status-download.success {
    color: #10b981;
  }

  .status-download.failed {
    color: #ef4444;
  }

  .spinner {
    width: 10px;
    height: 10px;
    border: 2px solid var(--vanta-text-dim);
    border-right-color: transparent;
    border-radius: 50%;
    animation: spin 0.75s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>

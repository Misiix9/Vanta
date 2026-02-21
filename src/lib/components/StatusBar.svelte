<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getVersion } from "@tauri-apps/api/app";

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
  let appVersion = $state("…");
  let unlisten: () => void;

  onMount(async () => {
    appVersion = await getVersion();

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
    <span class="status-badge">
      <img src="/32x32.png" alt="" class="status-badge-logo" />
      Vanta
    </span>
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
    <span class="status-version">v{appVersion}</span>
  </div>
</div>

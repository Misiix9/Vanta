<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getVersion } from "@tauri-apps/api/app";
  import type { BundleUpdateStatus, DownloadStatusPayload } from "$lib/types";

  let {
    resultCount = 0,
    searchTime = null,
    updateStatus = { status: "idle", message: "" },
    onCheckUpdate = undefined,
  }: {
    resultCount: number;
    searchTime: number | null;
    updateStatus?: BundleUpdateStatus | { status: "idle" | "checking"; message?: string };
    onCheckUpdate?: () => void;
  } = $props();

  let downloadStatus = $state<"idle" | "downloading" | "success" | "failed">(
    "idle",
  );
  let downloadError = $state("");
  let appVersion = $state("…");
  let unlisten: () => void;

  function formatUpdateLabel(
    status: BundleUpdateStatus | { status: "idle" | "checking"; message?: string },
  ): string {
    if (status.message) return status.message;
    switch (status.status) {
      case "checking":
        return "Checking…";
      case "not_installed":
        return "Bundle not installed";
      case "up_to_date":
        return "Bundle is up to date";
      case "update_available":
        return "Update available";
      case "error":
        return "Update check failed";
      default:
        return status.status;
    }
  }

  onMount(async () => {
    appVersion = await getVersion();

    unlisten = await listen<DownloadStatusPayload>(
      "download_status",
      (event) => {
        downloadStatus = event.payload.status as any;
        downloadError = event.payload.error || "";
        if (downloadStatus === "success" || downloadStatus === "failed") {
          setTimeout(() => {
            downloadStatus = "idle";
            downloadError = "";
          }, 3000);
        }
      },
    );
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
    {#if onCheckUpdate}
      <button
        class="status-update-btn"
        onclick={onCheckUpdate}
        disabled={updateStatus?.status === "checking"}
      >
        {#if updateStatus?.status === "checking"}
          <span class="spinner small"></span>
        {/if}
        Check update
      </button>
    {/if}
    {#if updateStatus && updateStatus.status !== "idle"}
      <div
        class={`status-update ${updateStatus.status}`}
        title={updateStatus.message}
      >
        {#if updateStatus.status === "checking"}
          <span class="spinner small"></span>
        {/if}
        {formatUpdateLabel(updateStatus)}
        {#if updateStatus.status === "update_available" && updateStatus.remote_version}
          <span class="status-update-version">
            {updateStatus.installed_version ?? "–"} → {updateStatus.remote_version}
          </span>
        {/if}
      </div>
    {/if}
    {#if downloadStatus !== "idle"}
      <div
        class="status-download"
        class:success={downloadStatus === "success"}
        class:failed={downloadStatus === "failed"}
        title={downloadError}
      >
        {#if downloadStatus === "downloading"}
          <span class="spinner"></span>
          Downloading
        {:else if downloadStatus === "success"}
          ✓ Downloaded
        {:else if downloadStatus === "failed"}
          ✗ Failed{#if downloadError}: {downloadError}{/if}
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


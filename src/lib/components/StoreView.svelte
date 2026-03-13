<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import type { StoreExtensionInfo, ExtensionEntry, ExtensionUpdateInfo } from "$lib/types";
  import type { ToastOptions } from "$lib/sdk/types";

  let {
    onClose,
    onToast,
  }: {
    onClose: () => void;
    onToast: (options: ToastOptions) => void;
  } = $props();

  let extensions: StoreExtensionInfo[] = $state([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let actionMessage = $state<string | null>(null);
  let actionLevel = $state<"info" | "success" | "error">("info");
  let installingSet = $state<Set<string>>(new Set());
  let rollbackingSet = $state<Set<string>>(new Set());
  let uninstallingSet = $state<Set<string>>(new Set());
  let updatesByName = $state<Record<string, string>>({});
  let selectedCategory = $state<string>("All");
  let ratingBusy = $state<string | null>(null);

  const categories = $derived([
    "All",
    ...Array.from(new Set(extensions.map((ext) => ext.category || "Uncategorized"))).sort(),
  ]);

  const visibleExtensions = $derived(
    selectedCategory === "All"
      ? extensions
      : extensions.filter((ext) => ext.category === selectedCategory),
  );

  const unlisteners: Array<() => void> = [];

  onMount(async () => {
    await fetchRegistry();

    unlisteners.push(
      await listen<ExtensionEntry[]>("extensions-changed", async () => {
        await refreshInstallState();
        await refreshAvailableUpdates();
      }),
    );
  });

  onDestroy(() => {
    for (const unlisten of unlisteners) unlisten();
  });

  async function fetchRegistry() {
    loading = true;
    error = null;
    actionMessage = null;
    try {
      extensions = await invoke<StoreExtensionInfo[]>("fetch_store_registry");
      await refreshAvailableUpdates();
    } catch (e) {
      error = String(e);
      console.error("Failed to fetch store registry:", e);
      onToast({ title: "Store Load Failed", message: String(e), type: "error" });
    } finally {
      loading = false;
    }
  }

  async function refreshAvailableUpdates() {
    try {
      const updates = await invoke<ExtensionUpdateInfo[]>("check_store_extension_updates");
      const next: Record<string, string> = {};
      for (const update of updates) next[update.name] = update.latest_version;
      updatesByName = next;
    } catch (e) {
      console.error("Failed to check extension updates:", e);
      onToast({ title: "Update Check Failed", message: String(e), type: "error" });
    }
  }

  async function refreshInstallState() {
    try {
      const installed = await invoke<ExtensionEntry[]>("get_extensions");
      const installedNames = new Set(installed.map((e) => e.manifest.name));
      extensions = extensions.map((ext) => ({
        ...ext,
        installed: installedNames.has(ext.name),
      }));
    } catch (e) {
      console.error("Failed to refresh install state:", e);
      onToast({ title: "Refresh Failed", message: String(e), type: "error" });
    }
  }

  async function installExtension(name: string) {
    installingSet = new Set([...installingSet, name]);
    actionMessage = null;
    try {
      await invoke("install_store_extension", { name });
      extensions = extensions.map((ext) =>
        ext.name === name ? { ...ext, installed: true } : ext,
      );
      actionLevel = "success";
      actionMessage = updatesByName[name]
        ? `Updated ${name} to v${updatesByName[name]}.`
        : `Installed ${name}.`;
      onToast({ title: updatesByName[name] ? "Extension Updated" : "Extension Installed", message: name, type: "success" });
      await refreshAvailableUpdates();
    } catch (e) {
      console.error(`Failed to install ${name}:`, e);
      actionLevel = "error";
      actionMessage = `Failed to install ${name}: ${String(e)}`;
      onToast({ title: "Install Failed", message: String(e), type: "error" });
    } finally {
      const next = new Set(installingSet);
      next.delete(name);
      installingSet = next;
    }
  }

  async function uninstallExtension(name: string) {
    uninstallingSet = new Set([...uninstallingSet, name]);
    actionMessage = null;
    try {
      await invoke("uninstall_extension", { name });
      extensions = extensions.map((ext) =>
        ext.name === name ? { ...ext, installed: false } : ext,
      );
      actionLevel = "success";
      actionMessage = `Uninstalled ${name}.`;
      onToast({ title: "Extension Uninstalled", message: name, type: "success" });
      await refreshAvailableUpdates();
    } catch (e) {
      console.error(`Failed to uninstall ${name}:`, e);
      actionLevel = "error";
      actionMessage = `Failed to uninstall ${name}: ${String(e)}`;
      onToast({ title: "Uninstall Failed", message: String(e), type: "error" });
    } finally {
      const next = new Set(uninstallingSet);
      next.delete(name);
      uninstallingSet = next;
    }
  }

  async function rollbackExtension(name: string) {
    rollbackingSet = new Set([...rollbackingSet, name]);
    actionMessage = null;
    try {
      const restoredVersion = await invoke<string>("rollback_store_extension", { name });
      actionLevel = "success";
      actionMessage = `Rolled back ${name} to v${restoredVersion}.`;
      onToast({ title: "Rollback Complete", message: `${name} -> v${restoredVersion}`, type: "success" });
      await refreshInstallState();
      await refreshAvailableUpdates();
    } catch (e) {
      console.error(`Failed to rollback ${name}:`, e);
      actionLevel = "error";
      actionMessage = `Failed to rollback ${name}: ${String(e)}`;
      onToast({ title: "Rollback Failed", message: String(e), type: "error" });
    } finally {
      const next = new Set(rollbackingSet);
      next.delete(name);
      rollbackingSet = next;
    }
  }

  async function submitRating(name: string, rating: number) {
    ratingBusy = `${name}:${rating}`;
    try {
      await invoke("submit_extension_rating", {
        name,
        rating,
        comment: null,
      });
      actionLevel = "success";
      actionMessage = `Opened rating form for ${name} (${rating} stars).`;
      onToast({ title: "Thanks For Rating", message: `${name} (${rating} stars)`, type: "success" });
    } catch (e) {
      console.error(`Failed to submit rating for ${name}:`, e);
      actionLevel = "error";
      actionMessage = `Failed to open rating form: ${String(e)}`;
      onToast({ title: "Rating Failed", message: String(e), type: "error" });
    } finally {
      ratingBusy = null;
    }
  }

  function isFontAwesome(icon: string | null | undefined): boolean {
    return !!icon && icon.startsWith("fa-");
  }

  function isInlineSvg(icon: string | null | undefined): boolean {
    return !!icon && icon.trim().startsWith("<svg");
  }

  function sanitizeInlineSvg(icon: string | null | undefined): string {
    if (!icon) return "";

    const trimmed = icon.trim();
    if (!trimmed.toLowerCase().startsWith("<svg")) return "";

    let cleaned = trimmed
      .replace(/<script[\s\S]*?<\/script>/gi, "")
      .replace(/<foreignObject[\s\S]*?<\/foreignObject>/gi, "")
      .replace(/\son[a-z]+\s*=\s*("[^"]*"|'[^']*'|[^\s>]+)/gi, "")
      .replace(/javascript:/gi, "")
      .replace(/\s(fill|stroke)\s*=\s*("([^"]*)"|'([^']*)')/gi, (_m, attr, _quoted, dq, sq) => {
        const value = (dq ?? sq ?? "").trim().toLowerCase();
        if (value === "none" || value === "currentcolor") {
          return ` ${attr}="${value}"`;
        }
        return ` ${attr}="currentColor"`;
      });

    if (!/\sviewBox=/i.test(cleaned)) {
      cleaned = cleaned.replace(/<svg/i, '<svg viewBox="0 0 24 24"');
    }

    return cleaned;
  }

  function formatRating(rating: number | null | undefined): string {
    if (typeof rating !== "number") return "N/A";
    return rating.toFixed(1);
  }

  function formatInstallCount(count: number): string {
    if (!Number.isFinite(count) || count <= 0) return "N/A";
    if (count >= 1_000_000) return `${(count / 1_000_000).toFixed(1)}M`;
    if (count >= 1_000) return `${(count / 1_000).toFixed(1)}k`;
    return `${count}`;
  }
</script>

<div class="store-root hub-window-root ext-surface-root" role="dialog" aria-modal="true" aria-label="Vanta Store">
  <div class="store-header hub-window-header ext-surface-header">
    <button class="back-btn btn-ghost icon-btn" onclick={onClose} aria-label="Close store">
      <i class="fa-solid fa-arrow-left"></i>
    </button>
    <h1>Vanta Store</h1>
    <button class="refresh-btn btn-ghost icon-btn" onclick={fetchRegistry} aria-label="Refresh store">
      <i class="fa-solid fa-rotate-right" class:spin={loading}></i>
    </button>
  </div>

  {#if actionMessage}
    <div class={`store-notice ${actionLevel}`} role="status" aria-live="polite">
      <span>{actionMessage}</span>
      <button class="store-notice-dismiss" onclick={() => (actionMessage = null)} aria-label="Dismiss message">
        <i class="fa-solid fa-xmark"></i>
      </button>
    </div>
  {/if}

  {#if loading}
    <div class="store-loading">
      <i class="fa-solid fa-spinner spin"></i>
      <span>Loading extensions...</span>
    </div>
  {:else if error}
    <div class="store-error">
      <i class="fa-solid fa-circle-exclamation"></i>
      <span>{error}</span>
      <button onclick={fetchRegistry}>Retry</button>
    </div>
  {:else if extensions.length === 0}
    <div class="store-empty">
      <i class="fa-solid fa-puzzle-piece"></i>
      <span>No extensions available yet.</span>
    </div>
  {:else}
    <div class="store-filters" role="tablist" aria-label="Store categories">
      {#each categories as category}
        <button
          class="store-filter btn-tag"
          class:active={selectedCategory === category}
          role="tab"
          aria-selected={selectedCategory === category}
          onclick={() => (selectedCategory = category)}
        >
          {category}
        </button>
      {/each}
    </div>

    {#if visibleExtensions.length === 0}
      <div class="store-empty">
        <i class="fa-solid fa-filter-circle-xmark"></i>
        <span>No extensions in this category yet.</span>
      </div>
    {:else}
    <div class="store-grid hub-window-scroll">
      {#each visibleExtensions as ext (ext.name)}
        <div class="ext-card ext-surface-card" class:installed={ext.installed}>
          <div class="ext-icon">
            {#if isInlineSvg(ext.icon)}
              <span class="icon-svg extension-svg" aria-label={ext.title}>
                {@html sanitizeInlineSvg(ext.icon)}
              </span>
            {:else if isFontAwesome(ext.icon)}
              <i class="{ext.icon} icon-fa"></i>
            {:else}
              <i class="fa-solid fa-puzzle-piece icon-fa"></i>
            {/if}
          </div>
          <div class="ext-info">
            <div class="ext-title-row">
              <h3>{ext.title}</h3>
              <span class={`trust-badge ${ext.trust_badge}`}>{ext.trust_badge}</span>
            </div>
            <p class="ext-desc">{ext.description}</p>
            <div class="ext-meta ext-meta-trust">
              {#if ext.safe}
                <span class="risk-badge risk-low">Safe</span>
              {/if}
              <span class={`risk-badge risk-${ext.permission_risk.toLowerCase()}`}>{ext.permission_risk} Risk</span>
            </div>
            <div class="ext-meta ext-meta-stats">
              <span class="ext-category">{ext.category}</span>
              <span class="ext-author">{ext.publisher}</span>
              <span class="ext-version">v{ext.version}</span>
              <span class="ext-rating"><i class="fa-solid fa-star"></i> {formatRating(ext.rating)}</span>
              <span class="ext-rating-count">({ext.rating_count || 0})</span>
              <span class="ext-installs"><i class="fa-solid fa-download"></i> {formatInstallCount(ext.install_count)}</span>
            </div>
            {#if ext.permissions.length > 0}
              <div class="ext-perms">
                {#each ext.permissions as perm}
                  <span class="perm-badge">{perm}</span>
                {/each}
              </div>
            {/if}
            {#if ext.changelog.length > 0}
              <div class="ext-changelog">Latest: {ext.changelog[0]}</div>
            {/if}
          </div>
          <div class="ext-actions ext-surface-actions">
            <div class="ext-rate-actions">
              {#each [1, 2, 3, 4, 5] as score}
                <button
                  class="btn-rate"
                  disabled={ratingBusy !== null}
                  onclick={() => submitRating(ext.name, score)}
                  aria-label={`Rate ${ext.title} ${score} stars`}
                >
                  {score}
                </button>
              {/each}
            </div>
            {#if ext.installed}
              <button
                class="btn-secondary"
                disabled={rollbackingSet.has(ext.name)}
                onclick={() => rollbackExtension(ext.name)}
              >
                {#if rollbackingSet.has(ext.name)}
                  <i class="fa-solid fa-spinner spin"></i> Rolling Back...
                {:else}
                  Rollback
                {/if}
              </button>
              <button
                class="btn-uninstall btn-secondary"
                disabled={uninstallingSet.has(ext.name)}
                onclick={() => uninstallExtension(ext.name)}
              >
                {#if uninstallingSet.has(ext.name)}
                  <i class="fa-solid fa-spinner spin"></i> Removing...
                {:else}
                  Uninstall
                {/if}
              </button>
              <span class="installed-badge">
                <i class="fa-solid fa-check"></i> Installed
              </span>
            {:else}
              <button
                class="btn-install btn-primary"
                disabled={installingSet.has(ext.name)}
                onclick={() => installExtension(ext.name)}
              >
                {#if installingSet.has(ext.name)}
                  <i class="fa-solid fa-spinner spin"></i> Installing...
                {:else}
                  <i class="fa-solid fa-download"></i> Install
                {/if}
              </button>
            {/if}
            {#if ext.installed && updatesByName[ext.name]}
              <button
                class="btn-install btn-primary"
                disabled={installingSet.has(ext.name)}
                onclick={() => installExtension(ext.name)}
              >
                {#if installingSet.has(ext.name)}
                  <i class="fa-solid fa-spinner spin"></i> Updating...
                {:else}
                  <i class="fa-solid fa-arrow-up"></i> Update to v{updatesByName[ext.name]}
                {/if}
              </button>
            {/if}
            {#if installingSet.has(ext.name) || uninstallingSet.has(ext.name)}
              <span class="ext-action-state">
                {installingSet.has(ext.name) ? "Installing package and assets..." : "Removing extension files..."}
              </span>
            {/if}
          </div>
        </div>
      {/each}
    </div>
    {/if}
  {/if}
</div>


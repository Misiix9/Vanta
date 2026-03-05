<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import type { StoreExtensionInfo, ExtensionEntry } from "$lib/types";

  let {
    onClose,
  }: {
    onClose: () => void;
  } = $props();

  let extensions: StoreExtensionInfo[] = $state([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let installingSet = $state<Set<string>>(new Set());
  let uninstallingSet = $state<Set<string>>(new Set());

  const unlisteners: Array<() => void> = [];

  onMount(async () => {
    await fetchRegistry();

    unlisteners.push(
      await listen<ExtensionEntry[]>("extensions-changed", async () => {
        await refreshInstallState();
      }),
    );
  });

  onDestroy(() => {
    for (const unlisten of unlisteners) unlisten();
  });

  async function fetchRegistry() {
    loading = true;
    error = null;
    try {
      extensions = await invoke<StoreExtensionInfo[]>("fetch_store_registry");
    } catch (e) {
      error = String(e);
      console.error("Failed to fetch store registry:", e);
    } finally {
      loading = false;
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
    }
  }

  async function installExtension(name: string) {
    installingSet = new Set([...installingSet, name]);
    try {
      await invoke("install_store_extension", { name });
      extensions = extensions.map((ext) =>
        ext.name === name ? { ...ext, installed: true } : ext,
      );
    } catch (e) {
      console.error(`Failed to install ${name}:`, e);
    } finally {
      const next = new Set(installingSet);
      next.delete(name);
      installingSet = next;
    }
  }

  async function uninstallExtension(name: string) {
    uninstallingSet = new Set([...uninstallingSet, name]);
    try {
      await invoke("uninstall_extension", { name });
      extensions = extensions.map((ext) =>
        ext.name === name ? { ...ext, installed: false } : ext,
      );
    } catch (e) {
      console.error(`Failed to uninstall ${name}:`, e);
    } finally {
      const next = new Set(uninstallingSet);
      next.delete(name);
      uninstallingSet = next;
    }
  }

  function isFontAwesome(icon: string | null | undefined): boolean {
    return !!icon && icon.startsWith("fa-");
  }

  function isInlineSvg(icon: string | null | undefined): boolean {
    return !!icon && icon.trim().startsWith("<svg");
  }

  function svgToDataUri(icon: string): string {
    return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(icon.trim())}`;
  }
</script>

<div class="store-root">
  <div class="store-header">
    <button class="back-btn" onclick={onClose} aria-label="Close store">
      <i class="fa-solid fa-arrow-left"></i>
    </button>
    <h1>Vanta Store</h1>
    <button class="refresh-btn" onclick={fetchRegistry} aria-label="Refresh store">
      <i class="fa-solid fa-rotate-right" class:spin={loading}></i>
    </button>
  </div>

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
    <div class="store-grid">
      {#each extensions as ext (ext.name)}
        <div class="ext-card" class:installed={ext.installed}>
          <div class="ext-icon">
            {#if isInlineSvg(ext.icon)}
              <img src={svgToDataUri(ext.icon!)} alt={ext.title} class="icon-svg" />
            {:else if isFontAwesome(ext.icon)}
              <i class="{ext.icon} icon-fa"></i>
            {:else}
              <i class="fa-solid fa-puzzle-piece icon-fa"></i>
            {/if}
          </div>
          <div class="ext-info">
            <h3>{ext.title}</h3>
            <p class="ext-desc">{ext.description}</p>
            <div class="ext-meta">
              <span class="ext-author">{ext.author}</span>
              <span class="ext-version">v{ext.version}</span>
            </div>
            {#if ext.permissions.length > 0}
              <div class="ext-perms">
                {#each ext.permissions as perm}
                  <span class="perm-badge">{perm}</span>
                {/each}
              </div>
            {/if}
          </div>
          <div class="ext-actions">
            {#if ext.installed}
              <button
                class="btn-uninstall"
                disabled={uninstallingSet.has(ext.name)}
                onclick={() => uninstallExtension(ext.name)}
              >
                {#if uninstallingSet.has(ext.name)}
                  <i class="fa-solid fa-spinner spin"></i>
                {:else}
                  Uninstall
                {/if}
              </button>
              <span class="installed-badge">
                <i class="fa-solid fa-check"></i> Installed
              </span>
            {:else}
              <button
                class="btn-install"
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
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .store-root {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    color: var(--vanta-text, #f5f5f5);
  }

  .store-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px 18px;
    border-bottom: 1px solid var(--vanta-border, rgba(255,255,255,0.08));
    flex-shrink: 0;
  }

  .store-header h1 {
    flex: 1;
    font-size: 1.1rem;
    font-weight: 600;
    margin: 0;
  }

  .back-btn,
  .refresh-btn {
    background: none;
    border: none;
    color: var(--vanta-text-dim, #888);
    cursor: pointer;
    padding: 6px 8px;
    border-radius: 6px;
    font-size: 0.95rem;
    transition: all 0.15s;
  }

  .back-btn:hover,
  .refresh-btn:hover {
    background: var(--vanta-surface, #0a0a0a);
    color: var(--vanta-text, #f5f5f5);
  }

  .store-loading,
  .store-error,
  .store-empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--vanta-text-dim, #888);
    font-size: 0.95rem;
  }

  .store-loading i,
  .store-error i,
  .store-empty i {
    font-size: 1.6rem;
    opacity: 0.6;
  }

  .store-error button {
    margin-top: 8px;
    padding: 6px 16px;
    border-radius: 6px;
    border: 1px solid var(--vanta-border, rgba(255,255,255,0.08));
    background: var(--vanta-surface, #0a0a0a);
    color: var(--vanta-text, #f5f5f5);
    cursor: pointer;
  }

  .store-grid {
    flex: 1;
    overflow-y: auto;
    padding: 14px 18px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .ext-card {
    display: flex;
    align-items: flex-start;
    gap: 14px;
    padding: 14px;
    border-radius: 10px;
    border: 1px solid var(--vanta-border, rgba(255,255,255,0.08));
    background: var(--vanta-surface, #0a0a0a);
    transition: border-color 0.15s;
  }

  .ext-card:hover {
    border-color: var(--vanta-accent, #7b35f0);
  }

  .ext-card.installed {
    border-color: color-mix(in srgb, var(--vanta-accent, #7b35f0) 40%, transparent);
  }

  .ext-icon {
    flex-shrink: 0;
    width: 48px;
    height: 48px;
    border-radius: 10px;
    background: color-mix(in srgb, var(--vanta-accent, #7b35f0) 12%, transparent);
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }

  .ext-icon img {
    width: 28px;
    height: 28px;
    object-fit: contain;
  }

  .ext-icon .icon-svg {
    filter: brightness(0) invert(1);
  }

  .ext-icon .icon-fa {
    font-size: 1.3rem;
    color: var(--vanta-accent, #7b35f0);
  }

  .ext-info {
    flex: 1;
    min-width: 0;
  }

  .ext-info h3 {
    font-size: 0.95rem;
    font-weight: 600;
    margin: 0 0 4px 0;
  }

  .ext-desc {
    font-size: 0.82rem;
    color: var(--vanta-text-dim, #888);
    margin: 0 0 6px 0;
    line-height: 1.3;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .ext-meta {
    display: flex;
    gap: 8px;
    font-size: 0.75rem;
    color: var(--vanta-text-dim, #888);
    opacity: 0.8;
  }

  .ext-perms {
    display: flex;
    gap: 4px;
    margin-top: 4px;
    flex-wrap: wrap;
  }

  .perm-badge {
    font-size: 0.68rem;
    padding: 1px 6px;
    border-radius: 4px;
    background: color-mix(in srgb, var(--vanta-accent, #7b35f0) 15%, transparent);
    color: var(--vanta-accent, #7b35f0);
    font-weight: 500;
  }

  .ext-actions {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 6px;
  }

  .btn-install {
    padding: 6px 16px;
    border-radius: 8px;
    border: none;
    background: var(--vanta-accent, #7b35f0);
    color: #fff;
    font-size: 0.82rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
    white-space: nowrap;
  }

  .btn-install:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .btn-install:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }

  .btn-uninstall {
    padding: 5px 12px;
    border-radius: 8px;
    border: 1px solid var(--vanta-border, rgba(255,255,255,0.08));
    background: transparent;
    color: var(--vanta-text-dim, #888);
    font-size: 0.78rem;
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-uninstall:hover:not(:disabled) {
    border-color: #e53e3e;
    color: #e53e3e;
  }

  .installed-badge {
    font-size: 0.72rem;
    color: var(--vanta-accent, #7b35f0);
    font-weight: 500;
  }

  .spin {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>

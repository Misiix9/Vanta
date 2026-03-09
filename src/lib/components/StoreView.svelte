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
  let actionMessage = $state<string | null>(null);
  let actionLevel = $state<"info" | "success" | "error">("info");
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
    actionMessage = null;
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
    actionMessage = null;
    try {
      await invoke("install_store_extension", { name });
      extensions = extensions.map((ext) =>
        ext.name === name ? { ...ext, installed: true } : ext,
      );
      actionLevel = "success";
      actionMessage = `Installed ${name}.`;
    } catch (e) {
      console.error(`Failed to install ${name}:`, e);
      actionLevel = "error";
      actionMessage = `Failed to install ${name}: ${String(e)}`;
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
    } catch (e) {
      console.error(`Failed to uninstall ${name}:`, e);
      actionLevel = "error";
      actionMessage = `Failed to uninstall ${name}: ${String(e)}`;
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
    <div class="store-grid">
      {#each extensions as ext (ext.name)}
        <div class="ext-card" class:installed={ext.installed}>
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


<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let {
    onClose,
    onOpenStore,
  }: {
    onClose: () => void;
    onOpenStore: () => void;
  } = $props();

  let templateName = $state("my-extension");
  let status = $state<string | null>(null);
  let busy = $state(false);

  async function createTemplate() {
    const normalized = templateName.trim().toLowerCase();
    if (!normalized) {
      status = "Extension name is required.";
      return;
    }

    busy = true;
    status = null;
    try {
      const path = await invoke<string>("create_extension_template", { name: normalized });
      status = `Template created at ${path}`;
      await invoke("open_path", { path });
    } catch (e) {
      status = `Template creation failed: ${String(e)}`;
    } finally {
      busy = false;
    }
  }
</script>

<div class="v2-shell v2-fill hub-window-root">
  <div class="v2-header hub-window-header">
    <div class="window-header-main">
      <span class="window-breadcrumb">Feature Hub</span>
      <h2>Extensions Hub</h2>
    </div>
    <div class="window-actions">
      <button class="link-btn btn-ghost" onclick={onClose}>Back</button>
      <button class="link-btn btn-secondary" onclick={onOpenStore}>Open Store</button>
    </div>
  </div>

  <div class="v2-scroll window-scroll">
    <div class="control-group control-group-block v2-card">
      <h3>Extension Template Generator</h3>
      <p class="hub-window-subtitle">Create a complete starter extension in one click.</p>
      <label>
        Extension Name
        <input type="text" bind:value={templateName} placeholder="my-extension" />
      </label>
      <div class="preset-row">
        <button class="preset-btn" disabled={busy} onclick={createTemplate}>
          {busy ? "Creating..." : "Create Template"}
        </button>
        <button class="preset-btn" onclick={onOpenStore}>Browse Existing Extensions</button>
      </div>
      {#if status}
        <div class="status-info">{status}</div>
      {/if}
    </div>

    <div class="control-group control-group-block v2-card">
      <h3>Template Workflow</h3>
      <ul class="hint-list">
        <li>1. Create template.</li>
        <li>2. Edit `manifest.json` and `dist/index.js`.</li>
        <li>3. Reload launcher and run your command.</li>
      </ul>
    </div>
  </div>
</div>

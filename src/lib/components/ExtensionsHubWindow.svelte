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

<div class="hub-window-root">
  <div class="hub-window-header ext-surface-header">
    <button class="link-btn btn-ghost" onclick={onClose}>Back</button>
    <h2>Extensions Hub</h2>
    <button class="link-btn btn-secondary" onclick={onOpenStore}>Open Store</button>
  </div>

  <div class="hub-window-scroll">
    <div class="control-group control-group-block ext-surface-card">
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

    <div class="control-group control-group-block ext-surface-card">
      <h3>Template Workflow</h3>
      <ul class="hint-list">
        <li>1. Create template.</li>
        <li>2. Edit `manifest.json` and `dist/index.js`.</li>
        <li>3. Reload launcher and run your command.</li>
      </ul>
    </div>
  </div>
</div>

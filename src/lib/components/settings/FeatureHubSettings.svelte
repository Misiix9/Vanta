<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";

    let {
        onOpenStore,
        onJumpToSection,
    }: {
        onOpenStore: () => void;
        onJumpToSection: (section: string) => void;
    } = $props();

    let templateName = $state("my-extension");
    let templateStatus = $state<string | null>(null);
    let templateBusy = $state(false);

    async function createExtensionTemplate() {
        const normalized = templateName.trim().toLowerCase();
        if (!normalized) { templateStatus = "Extension name is required."; return; }
        templateBusy = true; templateStatus = null;
        try {
            const path = await invoke<string>("create_extension_template", { name: normalized });
            templateStatus = `Template created at ${path}`;
            await invoke("open_path", { path });
        } catch (e) { templateStatus = `Template creation failed: ${String(e)}`; }
        finally { templateBusy = false; }
    }
</script>

<div class="control-group control-group-block">
    <h4>Start Here</h4>
    <p>Use these shortcuts to access major Vanta features quickly.</p>
    <div class="preset-row">
        <button class="preset-btn" onclick={() => onJumpToSection("Theme Profile")}>Theme/Profile Studio</button>
        <button class="preset-btn" onclick={() => onJumpToSection("Community")}>Sharing & Feedback</button>
        <button class="preset-btn" onclick={onOpenStore}>Extension Store</button>
        <button class="preset-btn" onclick={() => onJumpToSection("File Search")}>Search & Workflows</button>
    </div>
</div>

<div class="control-group control-group-block">
    <h4>Extension Template Starter</h4>
    <label>Extension Name <input type="text" bind:value={templateName} placeholder="my-extension" /></label>
    <div class="preset-row">
        <button class="preset-btn" disabled={templateBusy} onclick={createExtensionTemplate}>
            {templateBusy ? "Creating..." : "Create Template"}
        </button>
        <button class="preset-btn" onclick={() => onJumpToSection("Community")}>Open Sharing Snippets</button>
    </div>
    {#if templateStatus}<div class="status-info">{templateStatus}</div>{/if}
</div>

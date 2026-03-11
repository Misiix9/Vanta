<script lang="ts">
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
    import { LogicalSize } from "@tauri-apps/api/dpi";
    import type { VantaConfig } from "$lib/types";

    let {
        config = $bindable(),
        onSave,
        onOpenStore,
        onCommunityFeedOptInChange,
    }: {
        config: VantaConfig;
        onSave: () => void;
        onOpenStore: () => void;
        onCommunityFeedOptInChange: () => void;
    } = $props();

    function updateWindowSize() {
        if (config.window) {
            getCurrentWebviewWindow().setSize(
                new LogicalSize(config.window.width, config.window.height),
            );
        }
    }
</script>

<!-- Extensions -->
<div class="control-group control-group-block ext-surface-card">
    <h4>Extension Runtime</h4>
    <label>Extensions Directory
        <input type="text" bind:value={config.extensions.directory} oninput={onSave} readonly />
    </label>
    <label>Developer Mode
        <input type="checkbox" bind:checked={config.extensions.dev_mode} onchange={onSave} />
    </label>
</div>
<div class="control-group control-group-block ext-surface-card">
    <h4>Discovery</h4>
    <p class="hub-window-subtitle">Browse verified and community extensions with trust and permission risk metadata.</p>
    <div class="preset-row">
        <button class="preset-btn btn-secondary" onclick={onOpenStore}>Open Vanta Store</button>
    </div>
</div>

<hr style="border-color: var(--vanta-border, rgba(255,255,255,0.08)); margin: 12px 0;" />

<!-- Window -->
<h4>Window</h4>
<div class="control-group">
    <label>Width
        <input type="number" min="400" max="1920" bind:value={config.window.width}
            onchange={() => { onSave(); updateWindowSize(); }} />
    </label>
</div>
<div class="control-group">
    <label>Height
        <input type="number" min="300" max="1080" bind:value={config.window.height}
            onchange={() => { onSave(); updateWindowSize(); }} />
    </label>
</div>

<hr style="border-color: var(--vanta-border, rgba(255,255,255,0.08)); margin: 12px 0;" />

<!-- General -->
<h4>General</h4>
<div class="control-group">
    <label>Max Results
        <input type="number" min="1" max="20" bind:value={config.general.max_results} oninput={onSave} />
    </label>
</div>
<div class="control-group">
    <label>Hotkey (Restart Required)
        <input type="text" bind:value={config.general.hotkey} oninput={onSave} />
    </label>
</div>
<div class="control-group">
    <label>Opt In To Popular Workflows Feed
        <input type="checkbox" bind:checked={config.general.community_feed_opt_in} onchange={onCommunityFeedOptInChange} />
    </label>
</div>

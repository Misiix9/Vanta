<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import type { VantaConfig } from "$lib/types";

    let {
        config = $bindable(),
        onSave,
    }: {
        config: VantaConfig;
        onSave: () => void;
    } = $props();

    let availableApps: { name: string; exec: string }[] = $state([]);
    let includeGlobsText = $state("");
    let excludeGlobsText = $state("");
    let allowedExtsText = $state("");
    let rebuilding = $state(false);

    function parseList(val: string): string[] {
        return val.split(/[\n,]/).map((s) => s.trim()).filter((s) => s.length > 0);
    }

    function formatIndexedAt(): string {
        const ts = config.files.indexed_at;
        if (!ts) return "Not indexed yet";
        const date = new Date(ts);
        if (Number.isNaN(date.getTime())) return "Not indexed yet";
        const diffMs = Date.now() - date.getTime();
        const mins = Math.max(0, Math.floor(diffMs / 60000));
        if (mins < 1) return "Indexed just now";
        if (mins === 1) return "Indexed 1 minute ago";
        if (mins < 60) return `Indexed ${mins} minutes ago`;
        const hours = Math.floor(mins / 60);
        if (hours < 24) return `Indexed ${hours} hours ago`;
        return `Indexed on ${date.toLocaleString()}`;
    }

    function onIncludeGlobsChange(val: string) {
        includeGlobsText = val; config.files.include_globs = parseList(val); onSave();
    }
    function onExcludeGlobsChange(val: string) {
        excludeGlobsText = val; config.files.exclude_globs = parseList(val); onSave();
    }
    function onAllowedExtsChange(val: string) {
        allowedExtsText = val; config.files.allowed_extensions = parseList(val.toLowerCase()); onSave();
    }

    async function rebuildIndex() {
        rebuilding = true;
        try {
            const ts = await invoke<number | null>("rebuild_file_index");
            config.files.indexed_at = ts ?? null;
        } catch (e) { console.error("Failed to rebuild file index", e); }
        finally { rebuilding = false; }
    }

    onMount(async () => {
        try {
            const apps: any[] = await invoke("get_apps");
            availableApps = apps.map((a) => ({ name: a.name, exec: a.exec }));
        } catch (e) { console.error("Failed to fetch apps for settings:", e); }
    });

    $effect(() => {
        includeGlobsText = config.files.include_globs.join("\n");
        excludeGlobsText = config.files.exclude_globs.join("\n");
        allowedExtsText = config.files.allowed_extensions.join(", ");
    });
</script>

<div class="control-group">
    <label>Include Hidden Files
        <input type="checkbox" bind:checked={config.files.include_hidden} onchange={onSave} />
    </label>
</div>

<div class="control-group">
    <label>Include Globs
        <textarea rows="3" bind:value={includeGlobsText}
            oninput={(e) => onIncludeGlobsChange((e.target as HTMLTextAreaElement).value)}
            placeholder="e.g. **/*.md"></textarea>
    </label>
</div>

<div class="control-group">
    <label>Exclude Globs
        <textarea rows="3" bind:value={excludeGlobsText}
            oninput={(e) => onExcludeGlobsChange((e.target as HTMLTextAreaElement).value)}
            placeholder="e.g. **/node_modules/**"></textarea>
    </label>
</div>

<div class="control-group">
    <label>Allowed Extensions (comma or newline)
        <textarea rows="2" bind:value={allowedExtsText}
            oninput={(e) => onAllowedExtsChange((e.target as HTMLTextAreaElement).value)}
            placeholder="e.g. md, txt, rs"></textarea>
    </label>
</div>

<div class="control-group">
    <label>Type Filter
        <select class="vanta-select" bind:value={config.files.type_filter} onchange={onSave}>
            <option value="any">Any</option>
            <option value="file">Files Only</option>
            <option value="dir">Directories Only</option>
        </select>
    </label>
</div>

<div class="control-group">
    <label>Default File Manager
        <select class="vanta-select" bind:value={config.files.file_manager} onchange={onSave}>
            <option value="default">System Default</option>
            {#each availableApps as app}<option value={app.exec}>{app.name}</option>{/each}
        </select>
    </label>
</div>

<div class="control-group">
    <label>Default Text Editor
        <select class="vanta-select" bind:value={config.files.file_editor} onchange={onSave}>
            <option value="default">System Default</option>
            {#each availableApps as app}<option value={app.exec}>{app.name}</option>{/each}
        </select>
    </label>
</div>

<div class="control-group">
    <label>Open Documents in File Manager
        <input type="checkbox" bind:checked={config.files.open_docs_in_manager} onchange={onSave} />
    </label>
</div>

<div class="control-group">
    <label>Max Depth ({config.files.max_depth})
        <input type="range" min="1" max="10" step="1" bind:value={config.files.max_depth} onchange={onSave} />
    </label>
</div>

<div class="control-group control-group-inline">
    <div>{formatIndexedAt()}</div>
    <button class="link-btn" onclick={rebuildIndex} disabled={rebuilding}>
        {rebuilding ? "Rebuilding..." : "Rebuild Index"}
    </button>
</div>

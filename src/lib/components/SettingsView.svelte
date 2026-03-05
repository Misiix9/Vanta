<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
    import { LogicalSize } from "@tauri-apps/api/dpi";
    import type { VantaConfig, ThemeMeta, SearchDiagnostics } from "$lib/types";
    import { applyTheme } from "$lib/theme";

    let {
        config = $bindable(),
        availableThemes = [],
        onClose,
    }: {
        config: VantaConfig;
        availableThemes?: ThemeMeta[];
        onClose: () => void;
    } = $props();

    let saveTimeout: any = null;
    let availableApps: { name: string; exec: string }[] = $state([]);
    let diagnostics: SearchDiagnostics | null = $state(null);
    let includeGlobsText = $state("");
    let excludeGlobsText = $state("");
    let allowedExtsText = $state("");
    let rebuilding = $state(false);

    let activeSection = $state("Theme Profile");

    function toggleSection(name: string) {
        activeSection = activeSection === name ? "" : name;
    }


    async function loadApps() {
        try {
            const apps: any[] = await invoke("get_apps");
            availableApps = apps.map((a) => ({ name: a.name, exec: a.exec }));
        } catch (e) {
            console.error("Failed to fetch apps for settings:", e);
        }
    }

    async function loadDiagnostics() {
        try {
            diagnostics = await invoke<SearchDiagnostics>("get_search_diagnostics");
        } catch (e) {
            console.error("Failed to load diagnostics:", e);
        }
    }

    function debouncedSave() {
        // 1. Apply visual changes instantly via CSS variables
        applyTheme(config);

        // 2. Debounce the actual save to disk
        if (saveTimeout) clearTimeout(saveTimeout);
        saveTimeout = setTimeout(async () => {
            try {
                await invoke("save_config", { newConfig: config });
                console.log("Config saved to disk");
            } catch (e) {
                console.error("Failed to save config:", e);
            }
        }, 500);
    }

    function updateWindowSize() {
        if (config.window) {
            const win = getCurrentWebviewWindow();
            win.setSize(
                new LogicalSize(config.window.width, config.window.height),
            );
        }
    }

    function onThemeChange() {
        const selected = availableThemes.find(
            (t) => t.id === config.appearance.theme,
        );
        if (selected) {
            invoke("resize_window_for_theme", {
                width: selected.width,
                height: selected.height,
            });
        }
        debouncedSave();
    }

    // Initial apply
    onMount(() => {
        applyTheme(config);
        loadApps();
        loadDiagnostics();
    });

    $effect(() => {
        includeGlobsText = config.files.include_globs.join("\n");
        excludeGlobsText = config.files.exclude_globs.join("\n");
        allowedExtsText = config.files.allowed_extensions.join(", ");
    });

    function parseList(val: string): string[] {
        return val
            .split(/[\n,]/)
            .map((s) => s.trim())
            .filter((s) => s.length > 0);
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
        includeGlobsText = val;
        config.files.include_globs = parseList(val);
        debouncedSave();
    }

    function onExcludeGlobsChange(val: string) {
        excludeGlobsText = val;
        config.files.exclude_globs = parseList(val);
        debouncedSave();
    }

    function onAllowedExtsChange(val: string) {
        allowedExtsText = val;
        config.files.allowed_extensions = parseList(val.toLowerCase());
        debouncedSave();
    }

    async function rebuildIndex() {
        rebuilding = true;
        try {
            const ts = await invoke<number | null>("rebuild_file_index");
            config.files.indexed_at = ts ?? null;
        } catch (e) {
            console.error("Failed to rebuild file index", e);
        } finally {
            rebuilding = false;
        }
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === "Escape") {
            e.stopPropagation();
            onClose();
        }
    }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
    class="settings-panel"
    style="position:absolute;inset:0;z-index:50;background:var(--vanta-bg,#000);border-radius:var(--vanta-radius,24px);overflow:hidden;"
>
    <header>
        <h2>Settings</h2>
        <div class="actions">
            <button class="close-btn" onclick={onClose}>Done</button>
        </div>
    </header>

    <div class="sections">
        <!-- Theme Section -->
        <div class="accordion-item" class:active={activeSection === "Theme Profile"}>
            <button
                class="accordion-header"
                type="button"
                onclick={() => toggleSection("Theme Profile")}
                aria-expanded={activeSection === "Theme Profile"}
            >
                <svg class="accordion-icon" width="16" height="16" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>
                <h3>Theme Profile</h3>
            </button>
            {#if activeSection === "Theme Profile"}
                <div class="accordion-content">
<div class="control-group">
                <label>
                    Active Theme
                    <select
                        class="vanta-select"
                        bind:value={config.appearance.theme}
                        onchange={onThemeChange}
                    >
                        {#each availableThemes as theme}
                            <option value={theme.id}>{theme.name}</option>
                        {/each}
                    </select>
                </label>
            </div>

            <h3 style="margin-top: 1rem;">Colors (Overrides)</h3>

            <div class="control-group">
                <label
                    >Background Color
                    <input
                        type="color"
                        bind:value={config.appearance.colors.background}
                        oninput={debouncedSave}
                    />
                </label>
                <label
                    >Surface Color
                    <input
                        type="color"
                        bind:value={config.appearance.colors.surface}
                        oninput={debouncedSave}
                    />
                </label>
                <label
                    >Accent Color
                    <input
                        type="color"
                        bind:value={config.appearance.colors.accent}
                        oninput={debouncedSave}
                    />
                </label>
                <label
                    >Border Color
                    <input
                        type="text"
                        bind:value={config.appearance.colors.border}
                        oninput={debouncedSave}
                    />
                </label>
            </div>

            <div class="control-group">
                <label
                    >Opacity ({config.appearance.opacity})
                    <input
                        type="range"
                        min="0.1"
                        max="1"
                        step="0.05"
                        bind:value={config.appearance.opacity}
                        oninput={debouncedSave}
                    />
                </label>
            </div>

            <div class="control-group">
                <label
                    >Border Radius ({config.appearance.border_radius}px)
                    <input
                        type="range"
                        min="0"
                        max="50"
                        step="1"
                        bind:value={config.appearance.border_radius}
                        oninput={debouncedSave}
                    />
                </label>
            </div>

            <div class="control-group">
                <label
                    >Blur Radius ({config.appearance.blur_radius}px)
                    <input
                        type="range"
                        min="0"
                        max="100"
                        step="1"
                        bind:value={config.appearance.blur_radius}
                        oninput={debouncedSave}
                    />
                </label>
            </div>
                </div>
            {/if}
        </div>

        <!-- Extensions Section -->
        <div class="accordion-item" class:active={activeSection === "Extensions"}>
            <button
                class="accordion-header"
                type="button"
                onclick={() => toggleSection("Extensions")}
                aria-expanded={activeSection === "Extensions"}
            >
                <svg class="accordion-icon" width="16" height="16" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>
                <h3>Extensions</h3>
            </button>
            {#if activeSection === "Extensions"}
                <div class="accordion-content">
                    <div class="control-group">
                        <label>
                            Extensions Directory
                            <input
                                type="text"
                                bind:value={config.extensions.directory}
                                oninput={debouncedSave}
                                readonly
                            />
                        </label>
                    </div>
                    <div class="control-group">
                        <label>
                            Developer Mode
                            <input
                                type="checkbox"
                                bind:checked={config.extensions.dev_mode}
                                onchange={debouncedSave}
                            />
                        </label>
                    </div>
                </div>
            {/if}
        </div>

        <!-- Window Section -->
        <div class="accordion-item" class:active={activeSection === "Window"}>
            <button
                class="accordion-header"
                type="button"
                onclick={() => toggleSection("Window")}
                aria-expanded={activeSection === "Window"}
            >
                <svg class="accordion-icon" width="16" height="16" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>
                <h3>Window</h3>
            </button>
            {#if activeSection === "Window"}
                <div class="accordion-content">
<div class="control-group">
                <label
                    >Width
                    <input
                        type="number"
                        min="400"
                        max="1920"
                        bind:value={config.window.width}
                        onchange={() => {
                            debouncedSave();
                            updateWindowSize();
                        }}
                    />
                </label>
            </div>
            <div class="control-group">
                <label
                    >Height
                    <input
                        type="number"
                        min="300"
                        max="1080"
                        bind:value={config.window.height}
                        onchange={() => {
                            debouncedSave();
                            updateWindowSize();
                        }}
                    />
                </label>
            </div>
                </div>
            {/if}
        </div>

        <!-- General Section -->
        <div class="accordion-item" class:active={activeSection === "General"}>
            <button
                class="accordion-header"
                type="button"
                onclick={() => toggleSection("General")}
                aria-expanded={activeSection === "General"}
            >
                <svg class="accordion-icon" width="16" height="16" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>
                <h3>General</h3>
            </button>
            {#if activeSection === "General"}
                <div class="accordion-content">
<div class="control-group">
                <label
                    >Max Results
                    <input
                        type="number"
                        min="1"
                        max="20"
                        bind:value={config.general.max_results}
                        oninput={debouncedSave}
                    />
                </label>
            </div>
            <div class="control-group">
                <label
                    >Hotkey (Restart Required)
                    <input
                        type="text"
                        bind:value={config.general.hotkey}
                        oninput={debouncedSave}
                    />
                </label>
            </div>
                </div>
            {/if}
        </div>

        <!-- File Search Section -->
        <div class="accordion-item" class:active={activeSection === "File Search"}>
            <button
                class="accordion-header"
                type="button"
                onclick={() => toggleSection("File Search")}
                aria-expanded={activeSection === "File Search"}
            >
                <svg class="accordion-icon" width="16" height="16" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>
                <h3>File Search</h3>
            </button>
            {#if activeSection === "File Search"}
                <div class="accordion-content">
<div class="control-group">
                <label>
                    Include Hidden Files
                    <input
                        type="checkbox"
                        bind:checked={config.files.include_hidden}
                        onchange={debouncedSave}
                    />
                </label>
            </div>

            <div class="control-group">
                <label>
                    Include Globs
                    <textarea
                        rows="3"
                        bind:value={includeGlobsText}
                        oninput={(e) => onIncludeGlobsChange((e.target as HTMLTextAreaElement).value)}
                        placeholder="e.g. **/*.md"
                    ></textarea>
                </label>
            </div>

            <div class="control-group">
                <label>
                    Exclude Globs
                    <textarea
                        rows="3"
                        bind:value={excludeGlobsText}
                        oninput={(e) => onExcludeGlobsChange((e.target as HTMLTextAreaElement).value)}
                        placeholder="e.g. **/node_modules/**"
                    ></textarea>
                </label>
            </div>

            <div class="control-group">
                <label>
                    Allowed Extensions (comma or newline)
                    <textarea
                        rows="2"
                        bind:value={allowedExtsText}
                        oninput={(e) => onAllowedExtsChange((e.target as HTMLTextAreaElement).value)}
                        placeholder="e.g. md, txt, rs"
                    ></textarea>
                </label>
            </div>

            <div class="control-group">
                <label>
                    Type Filter
                    <select
                        class="vanta-select"
                        bind:value={config.files.type_filter}
                        onchange={debouncedSave}
                    >
                        <option value="any">Any</option>
                        <option value="file">Files Only</option>
                        <option value="dir">Directories Only</option>
                    </select>
                </label>
            </div>

            <div class="control-group">
                <label>
                    Default File Manager
                    <select
                        class="vanta-select"
                        bind:value={config.files.file_manager}
                        onchange={debouncedSave}
                    >
                        <option value="default">System Default</option>
                        {#each availableApps as app}
                            <option value={app.exec}>{app.name}</option>
                        {/each}
                    </select>
                </label>
            </div>

            <div class="control-group">
                <label>
                    Default Text Editor
                    <select
                        class="vanta-select"
                        bind:value={config.files.file_editor}
                        onchange={debouncedSave}
                    >
                        <option value="default">System Default</option>
                        {#each availableApps as app}
                            <option value={app.exec}>{app.name}</option>
                        {/each}
                    </select>
                </label>
            </div>

            <div class="control-group">
                <label>
                    Open Documents in File Manager
                    <input
                        type="checkbox"
                        bind:checked={config.files.open_docs_in_manager}
                        onchange={debouncedSave}
                    />
                </label>
            </div>

            <div class="control-group">
                <label>
                    Max Depth ({config.files.max_depth})
                    <input
                        type="range"
                        min="1"
                        max="10"
                        step="1"
                        bind:value={config.files.max_depth}
                        onchange={debouncedSave}
                    />
                </label>
            </div>

            <div class="control-group" style="gap: 12px; align-items: center;">
                <div>{formatIndexedAt()}</div>
                <button class="link-btn" onclick={rebuildIndex} disabled={rebuilding}>
                    {rebuilding ? "Rebuilding..." : "Rebuild Index"}
                </button>
            </div>
                </div>
            {/if}
        </div>

        <!-- Search Ranking Section -->
        <div class="accordion-item" class:active={activeSection === "Search Ranking"}>
            <button
                class="accordion-header"
                type="button"
                onclick={() => toggleSection("Search Ranking")}
                aria-expanded={activeSection === "Search Ranking"}
            >
                <svg class="accordion-icon" width="16" height="16" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>
                <h3>Search Ranking</h3>
            </button>
            {#if activeSection === "Search Ranking"}
                <div class="accordion-content">
<div class="control-group">
                <label>
                    Applications Enabled
                    <input
                        type="checkbox"
                        bind:checked={config.search.applications.enabled}
                        onchange={debouncedSave}
                    />
                </label>
                <label>
                    Application Weight ({config.search.applications.weight}%)
                    <input
                        type="range"
                        min="10"
                        max="300"
                        step="5"
                        bind:value={config.search.applications.weight}
                        onchange={debouncedSave}
                    />
                </label>
            </div>

            <div class="control-group">
                <label>
                    Windows Enabled
                    <input
                        type="checkbox"
                        bind:checked={config.search.windows.enabled}
                        onchange={debouncedSave}
                    />
                </label>
                <label>
                    Window Weight ({config.search.windows.weight}%)
                    <input
                        type="range"
                        min="10"
                        max="300"
                        step="5"
                        bind:value={config.search.windows.weight}
                        onchange={debouncedSave}
                    />
                </label>
            </div>

            <div class="control-group">
                <label>
                    Calculator Enabled
                    <input
                        type="checkbox"
                        bind:checked={config.search.calculator.enabled}
                        onchange={debouncedSave}
                    />
                </label>
                <label>
                    Calculator Weight ({config.search.calculator.weight}%)
                    <input
                        type="range"
                        min="10"
                        max="300"
                        step="5"
                        bind:value={config.search.calculator.weight}
                        onchange={debouncedSave}
                    />
                </label>
            </div>

            <div class="control-group">
                <label>
                    File Search Enabled
                    <input
                        type="checkbox"
                        bind:checked={config.search.files.enabled}
                        onchange={debouncedSave}
                    />
                </label>
                <label>
                    File Weight ({config.search.files.weight}%)
                    <input
                        type="range"
                        min="10"
                        max="300"
                        step="5"
                        bind:value={config.search.files.weight}
                        onchange={debouncedSave}
                    />
                </label>
            </div>
                </div>
            {/if}
        </div>

        <!-- Diagnostics Section -->
        <div class="accordion-item" class:active={activeSection === "Diagnostics"}>
            <button
                class="accordion-header"
                type="button"
                onclick={() => toggleSection("Diagnostics")}
                aria-expanded={activeSection === "Diagnostics"}
            >
                <svg class="accordion-icon" width="16" height="16" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>
                <h3>Diagnostics</h3>
            </button>
            {#if activeSection === "Diagnostics"}
                <div class="accordion-content">
<div class="control-group">
                <button class="close-btn" onclick={loadDiagnostics}>Refresh Metrics</button>
            </div>
            {#if diagnostics}
                <div class="control-group">
                    <label>
                        Search Calls
                        <input type="text" value={`${diagnostics.search.calls}`} readonly />
                    </label>
                    <label>
                        Search Avg (ms)
                        <input type="text" value={diagnostics.search.avg_ms.toFixed(2)} readonly />
                    </label>
                    <label>
                        Search Max (ms)
                        <input type="text" value={`${diagnostics.search.max_ms}`} readonly />
                    </label>
                </div>
                <div class="control-group">
                    <label>
                        Suggestion Calls
                        <input type="text" value={`${diagnostics.suggestions.calls}`} readonly />
                    </label>
                    <label>
                        Suggestion Avg (ms)
                        <input type="text" value={diagnostics.suggestions.avg_ms.toFixed(2)} readonly />
                    </label>
                    <label>
                        Launch Avg (ms)
                        <input type="text" value={diagnostics.launch.avg_ms.toFixed(2)} readonly />
                    </label>
                </div>
            {/if}
                </div>
            {/if}
        </div>
    </div>
</div>

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
        <section>
            <h3>Theme Profile</h3>

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
        </section>

        <!-- Window Section -->
        <section>
            <h3>Window</h3>
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
        </section>

        <!-- General Section -->
        <section>
            <h3>General</h3>
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
        </section>

        <!-- File Search Section -->
        <section>
            <h3>File Search</h3>
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
        </section>

        <!-- Search Ranking Section -->
        <section>
            <h3>Search Ranking</h3>

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
        </section>

        <!-- Diagnostics Section -->
        <section>
            <h3>Diagnostics</h3>
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
        </section>
    </div>
</div>

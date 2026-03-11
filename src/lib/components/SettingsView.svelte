<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
    import { LogicalSize } from "@tauri-apps/api/dpi";
    import type { VantaConfig, ThemeMeta } from "$lib/types";
    import { applyTheme } from "$lib/theme";
    import ThemeSettings from "$lib/components/settings/ThemeSettings.svelte";
    import CommunitySettings from "$lib/components/settings/CommunitySettings.svelte";
    import FeatureHubSettings from "$lib/components/settings/FeatureHubSettings.svelte";
    import AccessibilitySettings from "$lib/components/settings/AccessibilitySettings.svelte";
    import FileSearchSettings from "$lib/components/settings/FileSearchSettings.svelte";
    import DiagnosticsSettings from "$lib/components/settings/DiagnosticsSettings.svelte";

    let {
        config = $bindable(),
        availableThemes = [],
        initialSection = null,
        onOpenStore = () => {},
        onClose,
    }: {
        config: VantaConfig;
        availableThemes?: ThemeMeta[];
        initialSection?: string | null;
        onOpenStore?: () => void;
        onClose: () => void;
    } = $props();

    let saveTimeout: any = null;
    let activeSection = $state("Feature Hub");
    let lastInitialSection = $state<string | null>(null);
    let sectionQuery = $state("");
    let sectionSearchStatus = $state<string | null>(null);

    const sectionAliases: Record<string, string> = {
        "theme": "Theme Profile", "appearance": "Theme Profile",
        "extensions": "Extensions", "window": "Window", "general": "General",
        "community": "Community", "feature-hub": "Feature Hub", "feature hub": "Feature Hub",
        "accessibility": "Accessibility", "file-search": "File Search", "file search": "File Search",
        "diagnostics": "Diagnostics",
    };

    const sectionSearchIndex: Array<{ section: string; terms: string[] }> = [
        { section: "Feature Hub", terms: ["start", "discover", "template", "shortcuts"] },
        { section: "Theme Profile", terms: ["appearance", "theme", "colors", "studio"] },
        { section: "General", terms: ["hotkey", "results", "community feed"] },
        { section: "File Search", terms: ["files", "globs", "index", "extensions"] },
        { section: "Accessibility", terms: ["motion", "text", "spacing", "compact", "relaxed"] },
        { section: "Community", terms: ["feedback", "snippets", "workflows", "vote"] },
        { section: "Extensions", terms: ["directory", "dev mode"] },
        { section: "Window", terms: ["width", "height", "size"] },
        { section: "Diagnostics", terms: ["metrics", "health", "recovery", "support bundle"] },
    ];

    const sections = [
        "Feature Hub", "Theme Profile", "Extensions", "Window",
        "General", "Community", "Accessibility", "File Search", "Diagnostics",
    ];

    function toggleSection(name: string) { activeSection = activeSection === name ? "" : name; }
    function jumpToSection(name: string) { activeSection = name; }

    function normalizeSection(section: string | null | undefined): string | null {
        if (!section) return null;
        const trimmed = section.trim();
        if (!trimmed) return null;
        return sectionAliases[trimmed.toLowerCase()] ?? trimmed;
    }

    function runSectionSearch() {
        const q = sectionQuery.trim().toLowerCase();
        if (!q) { sectionSearchStatus = null; return; }
        const hit = sectionSearchIndex.find((e) =>
            e.section.toLowerCase().includes(q) || e.terms.some((t) => t.includes(q)),
        );
        if (hit) { activeSection = hit.section; sectionSearchStatus = `Jumped to ${hit.section}`; }
        else { sectionSearchStatus = "No matching section"; }
    }

    function debouncedSave() {
        applyTheme(config);
        if (saveTimeout) clearTimeout(saveTimeout);
        saveTimeout = setTimeout(async () => {
            try { await invoke("save_config", { newConfig: config }); }
            catch (e) { console.error("Failed to save config:", e); }
        }, 500);
    }

    function updateWindowSize() {
        if (config.window) {
            getCurrentWebviewWindow().setSize(new LogicalSize(config.window.width, config.window.height));
        }
    }

    async function onCommunityFeedOptInChange() {
        debouncedSave();
    }

    function flushAndClose() {
        if (saveTimeout) clearTimeout(saveTimeout);
        invoke("save_config", { newConfig: config }).catch(console.error);
        onClose();
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === "Escape") { e.stopPropagation(); flushAndClose(); }
    }

    onMount(() => {
        if (config.general.community_feed_opt_in === undefined) config.general.community_feed_opt_in = false;
        if (config.search.show_explain_panel === undefined) config.search.show_explain_panel = true;
        applyTheme(config);
    });

    $effect(() => {
        const norm = normalizeSection(initialSection);
        if (norm && norm !== lastInitialSection) { activeSection = norm; lastInitialSection = norm; }
    });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="settings-panel v2-panel v2-stack" role="dialog" aria-modal="true" aria-label="Settings">
    <header class="settings-header sticky">
        <div class="settings-header-main">
            <h2>Settings</h2>
            <input type="text" class="settings-section-search" bind:value={sectionQuery}
                oninput={runSectionSearch} placeholder="Jump to section (theme, files, diagnostics...)"
                aria-label="Search settings sections" />
        </div>
        <div class="actions">
            <button class="close-btn" onclick={flushAndClose}>Done</button>
        </div>
    </header>

    {#if sectionSearchStatus}
        <div class="status-info settings-search-status">{sectionSearchStatus}</div>
    {/if}

    <div class="sections v2-stack density-comfortable">
        {#each sections as name}
            <div class="accordion-item" class:active={activeSection === name}>
                <button class="accordion-header" type="button" onclick={() => toggleSection(name)}
                    aria-expanded={activeSection === name}>
                    <svg class="accordion-icon" width="16" height="16" xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
                        stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>
                    <h3>{name}</h3>
                </button>
                {#if activeSection === name}
                    <div class="accordion-content">
                        {#if name === "Feature Hub"}
                            <FeatureHubSettings {onOpenStore} onJumpToSection={jumpToSection} />

                        {:else if name === "Theme Profile"}
                            <ThemeSettings bind:config {availableThemes} onSave={debouncedSave} />

                        {:else if name === "Extensions"}
                            <div class="control-group control-group-block ext-surface-card">
                                <h4>Extension Runtime</h4>
                                <label>Extensions Directory
                                    <input type="text" bind:value={config.extensions.directory} oninput={debouncedSave} readonly />
                                </label>
                                <label>Developer Mode
                                    <input type="checkbox" bind:checked={config.extensions.dev_mode} onchange={debouncedSave} />
                                </label>
                            </div>
                            <div class="control-group control-group-block ext-surface-card">
                                <h4>Discovery</h4>
                                <p class="hub-window-subtitle">Browse verified and community extensions with trust and permission risk metadata.</p>
                                <div class="preset-row">
                                    <button class="preset-btn btn-secondary" onclick={onOpenStore}>Open Vanta Store</button>
                                    <button class="preset-btn" onclick={() => jumpToSection("Community")}>Shareable Snippets</button>
                                </div>
                            </div>

                        {:else if name === "Window"}
                            <div class="control-group">
                                <label>Width
                                    <input type="number" min="400" max="1920" bind:value={config.window.width}
                                        onchange={() => { debouncedSave(); updateWindowSize(); }} />
                                </label>
                            </div>
                            <div class="control-group">
                                <label>Height
                                    <input type="number" min="300" max="1080" bind:value={config.window.height}
                                        onchange={() => { debouncedSave(); updateWindowSize(); }} />
                                </label>
                            </div>

                        {:else if name === "General"}
                            <div class="control-group">
                                <label>Max Results
                                    <input type="number" min="1" max="20" bind:value={config.general.max_results} oninput={debouncedSave} />
                                </label>
                            </div>
                            <div class="control-group">
                                <label>Hotkey (Restart Required)
                                    <input type="text" bind:value={config.general.hotkey} oninput={debouncedSave} />
                                </label>
                            </div>
                            <div class="control-group">
                                <label>Opt In To Popular Workflows Feed
                                    <input type="checkbox" bind:checked={config.general.community_feed_opt_in}
                                        onchange={onCommunityFeedOptInChange} />
                                </label>
                            </div>

                        {:else if name === "Community"}
                            <CommunitySettings bind:config onSave={debouncedSave} />

                        {:else if name === "Accessibility"}
                            <AccessibilitySettings bind:config onSave={debouncedSave} />

                        {:else if name === "File Search"}
                            <FileSearchSettings bind:config onSave={debouncedSave} />

                        {:else if name === "Diagnostics"}
                            <DiagnosticsSettings />
                        {/if}
                    </div>
                {/if}
            </div>
        {/each}
    </div>
</div>

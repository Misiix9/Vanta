<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
    import { LogicalSize } from "@tauri-apps/api/dpi";
    import type {
        VantaConfig,
        ThemeMeta,
        SearchDiagnostics,
        HealthDashboard,
        RecoveryHint,
        SupportBundleReport,
        CommunityFeedbackSummary,
        CommunityFeedbackReceipt,
        PopularWorkflowFeedEntry,
        WorkflowMacro,
        ProfilesConfig,
    } from "$lib/types";
    import { applyTheme } from "$lib/theme";

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
    let availableApps: { name: string; exec: string }[] = $state([]);
    let diagnostics: SearchDiagnostics | null = $state(null);
    let healthDashboard: HealthDashboard | null = $state(null);
    let recoveryHints: RecoveryHint[] = $state([]);
    let supportBundle: SupportBundleReport | null = $state(null);
    let supportBusy = $state(false);
    let communitySummary: CommunityFeedbackSummary | null = $state(null);
    let communityFeed: PopularWorkflowFeedEntry[] = $state([]);
    let availableWorkflowIds: string[] = $state([]);
    let availableProfileIds: string[] = $state([]);
    let feedbackMessage = $state("");
    let feedbackContact = $state("");
    let selectedRoadmapTopic = $state("popular-workflows-feed");
    let snippetKind = $state<"workflow" | "profile" | "theme">("workflow");
    let snippetTargetId = $state("");
    let snippetPayload = $state("");
    let snippetStatus = $state<string | null>(null);
    let templateName = $state("my-extension");
    let templateStatus = $state<string | null>(null);
    let templateBusy = $state(false);
    let feedbackStatus = $state<string | null>(null);
    let communityBusy = $state(false);
    let includeGlobsText = $state("");
    let excludeGlobsText = $state("");
    let allowedExtsText = $state("");
    let rebuilding = $state(false);
    let themeStudioText = $state("");
    let themeStudioStatus = $state<string | null>(null);

    const DEFAULT_APPEARANCE = {
        blur_radius: 40,
        opacity: 0.85,
        border_radius: 24,
        theme: "default",
        colors: {
            background: "#000000",
            surface: "#0A0A0A",
            accent: "#FFFFFF",
            accent_glow: "rgba(255, 255, 255, 0.25)",
            text_primary: "#F5F5F5",
            text_secondary: "#888888",
            border: "rgba(255, 255, 255, 0.08)",
        },
    } as const;

    let activeSection = $state("Feature Hub");
    let lastInitialSection = $state<string | null>(null);
    const roadmapTopics = [
        { id: "shareable-snippets", label: "Shareable snippets" },
        { id: "popular-workflows-feed", label: "Popular workflows feed" },
        { id: "in-app-feedback-channel", label: "In-app feedback channel" },
        { id: "adaptive-intent-engine", label: "Adaptive intent engine (Phase 20)" },
    ];

    function toggleSection(name: string) {
        activeSection = activeSection === name ? "" : name;
    }

    function jumpToSection(name: string) {
        activeSection = name;
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

    async function loadHealthDashboard() {
        try {
            healthDashboard = await invoke<HealthDashboard>("get_health_dashboard");
        } catch (e) {
            console.error("Failed to load health dashboard:", e);
        }
    }

    async function loadRecoveryHints() {
        try {
            recoveryHints = await invoke<RecoveryHint[]>("get_recovery_hints");
        } catch (e) {
            console.error("Failed to load recovery hints:", e);
        }
    }

    async function buildSupportBundle() {
        supportBusy = true;
        try {
            supportBundle = await invoke<SupportBundleReport>("create_support_bundle", {
                outputPath: null,
            });
            await loadHealthDashboard();
            await loadRecoveryHints();
        } catch (e) {
            console.error("Failed to create support bundle:", e);
        } finally {
            supportBusy = false;
        }
    }

    async function loadCommunitySummary() {
        try {
            communitySummary = await invoke<CommunityFeedbackSummary>("get_community_feedback_summary");
        } catch (e) {
            console.error("Failed to load community summary:", e);
        }
    }

    async function loadPopularWorkflowsFeed() {
        if (!config.general.community_feed_opt_in) {
            communityFeed = [];
            return;
        }
        try {
            communityFeed = await invoke<PopularWorkflowFeedEntry[]>("get_popular_workflows_feed");
        } catch (e) {
            feedbackStatus = `Failed to load popular feed: ${String(e)}`;
        }
    }

    async function loadWorkflowTargets() {
        try {
            const macros = await invoke<WorkflowMacro[]>("get_workflows");
            availableWorkflowIds = macros.map((m) => m.id).sort();
            if (!snippetTargetId && availableWorkflowIds.length > 0) {
                snippetTargetId = availableWorkflowIds[0];
            }
        } catch (e) {
            console.error("Failed to load workflow targets:", e);
        }

        try {
            const profiles = await invoke<ProfilesConfig>("get_profiles");
            availableProfileIds = profiles.entries.map((p) => p.id).sort();
            if (snippetKind === "profile" && availableProfileIds.length > 0 && !snippetTargetId) {
                snippetTargetId = availableProfileIds[0];
            }
        } catch (e) {
            console.error("Failed to load profile targets:", e);
        }
    }

    async function installPopularWorkflow(id: string) {
        communityBusy = true;
        feedbackStatus = null;
        try {
            await invoke("install_popular_workflow", { workflowId: id });
            feedbackStatus = `Installed popular workflow '${id}'.`;
            await loadWorkflowTargets();
        } catch (e) {
            feedbackStatus = `Failed to install workflow: ${String(e)}`;
        } finally {
            communityBusy = false;
        }
    }

    function selectedSnippetTarget(): string | null {
        if (snippetKind === "theme") return null;
        return snippetTargetId.trim() || null;
    }

    async function exportCommunitySnippet() {
        snippetStatus = null;
        try {
            snippetPayload = await invoke<string>("export_community_snippet", {
                kind: snippetKind,
                targetId: selectedSnippetTarget(),
            });
            snippetStatus = `Exported ${snippetKind} snippet.`;
        } catch (e) {
            snippetStatus = `Export failed: ${String(e)}`;
        }
    }

    async function importCommunitySnippet() {
        if (!snippetPayload.trim()) {
            snippetStatus = "Paste a snippet payload first.";
            return;
        }

        snippetStatus = null;
        try {
            const message = await invoke<string>("import_community_snippet", {
                snippetJson: snippetPayload,
            });
            snippetStatus = message;
            await loadWorkflowTargets();
        } catch (e) {
            snippetStatus = `Import failed: ${String(e)}`;
        }
    }

    function onSnippetKindChange(nextKind: "workflow" | "profile" | "theme") {
        snippetKind = nextKind;
        if (nextKind === "workflow") {
            snippetTargetId = availableWorkflowIds[0] || "";
        } else if (nextKind === "profile") {
            snippetTargetId = availableProfileIds[0] || "";
        } else {
            snippetTargetId = "";
        }
    }

    async function onCommunityFeedOptInChange() {
        debouncedSave();
        await loadPopularWorkflowsFeed();
    }

    async function createExtensionTemplateFromHub() {
        const normalized = templateName.trim().toLowerCase();
        if (!normalized) {
            templateStatus = "Extension name is required.";
            return;
        }

        templateBusy = true;
        templateStatus = null;
        try {
            const path = await invoke<string>("create_extension_template", {
                name: normalized,
            });
            templateStatus = `Template created at ${path}`;
            await invoke("open_path", { path });
        } catch (e) {
            templateStatus = `Template creation failed: ${String(e)}`;
        } finally {
            templateBusy = false;
        }
    }

    async function submitCommunityFeedback() {
        if (!feedbackMessage.trim()) {
            feedbackStatus = "Please add feedback before submitting.";
            return;
        }

        communityBusy = true;
        feedbackStatus = null;
        try {
            const receipt = await invoke<CommunityFeedbackReceipt>("submit_community_feedback", {
                message: feedbackMessage,
                contact: feedbackContact.trim() || null,
                roadmapTopic: selectedRoadmapTopic || null,
            });
            feedbackMessage = "";
            feedbackStatus = `Thanks, feedback recorded (${receipt.id}).`;
            await loadCommunitySummary();
        } catch (e) {
            feedbackStatus = `Failed to submit feedback: ${String(e)}`;
        } finally {
            communityBusy = false;
        }
    }

    async function voteRoadmapTopic(topic: string) {
        communityBusy = true;
        feedbackStatus = null;
        try {
            await invoke("vote_roadmap_item", { topic });
            selectedRoadmapTopic = topic;
            feedbackStatus = `Vote recorded for ${topic}.`;
            await loadCommunitySummary();
        } catch (e) {
            feedbackStatus = `Failed to vote: ${String(e)}`;
        } finally {
            communityBusy = false;
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
        if (config.general.community_feed_opt_in === undefined) {
            config.general.community_feed_opt_in = false;
        }
        if (config.search.show_explain_panel === undefined) {
            config.search.show_explain_panel = true;
        }
        applyTheme(config);
        loadApps();
        loadDiagnostics();
        loadHealthDashboard();
        loadRecoveryHints();
        loadCommunitySummary();
        loadWorkflowTargets();
        loadPopularWorkflowsFeed();
    });

    $effect(() => {
        if (initialSection && initialSection !== lastInitialSection) {
            activeSection = initialSection;
            lastInitialSection = initialSection;
        }
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

    function applySearchPreset(preset: "developer" | "creator" | "minimal") {
        if (preset === "developer") {
            config.search.applications.weight = 120;
            config.search.windows.weight = 150;
            config.search.files.weight = 170;
            config.search.calculator.weight = 90;
            config.search.windows.enabled = true;
            config.search.files.enabled = true;
            config.search.calculator.enabled = true;
        } else if (preset === "creator") {
            config.search.applications.weight = 160;
            config.search.windows.weight = 120;
            config.search.files.weight = 95;
            config.search.calculator.weight = 130;
            config.search.windows.enabled = true;
            config.search.files.enabled = true;
            config.search.calculator.enabled = true;
        } else {
            config.search.applications.weight = 180;
            config.search.windows.weight = 80;
            config.search.files.weight = 70;
            config.search.calculator.weight = 100;
            config.search.windows.enabled = false;
            config.search.files.enabled = true;
            config.search.calculator.enabled = true;
        }

        debouncedSave();
    }

    function applyThemePreset(preset: "minimal" | "vivid" | "high-contrast" | "compact") {
        if (preset === "minimal") {
            config.appearance.colors.background = "#070707";
            config.appearance.colors.surface = "#111111";
            config.appearance.colors.accent = "#DADADA";
            config.appearance.colors.accent_glow = "rgba(218, 218, 218, 0.22)";
            config.appearance.colors.text_primary = "#F2F2F2";
            config.appearance.colors.text_secondary = "#A0A0A0";
            config.appearance.colors.border = "rgba(255, 255, 255, 0.1)";
        } else if (preset === "vivid") {
            config.appearance.colors.background = "#0B1020";
            config.appearance.colors.surface = "#151B33";
            config.appearance.colors.accent = "#4DE3C3";
            config.appearance.colors.accent_glow = "rgba(77, 227, 195, 0.35)";
            config.appearance.colors.text_primary = "#ECF5FF";
            config.appearance.colors.text_secondary = "#92A2C5";
            config.appearance.colors.border = "rgba(77, 227, 195, 0.22)";
        } else if (preset === "high-contrast") {
            config.appearance.colors.background = "#000000";
            config.appearance.colors.surface = "#0A0A0A";
            config.appearance.colors.accent = "#FFD400";
            config.appearance.colors.accent_glow = "rgba(255, 212, 0, 0.32)";
            config.appearance.colors.text_primary = "#FFFFFF";
            config.appearance.colors.text_secondary = "#C2C2C2";
            config.appearance.colors.border = "rgba(255, 255, 255, 0.2)";
        } else {
            config.window.width = 620;
            config.window.height = 380;
            config.appearance.opacity = 0.9;
            config.appearance.blur_radius = 32;
            updateWindowSize();
        }

        themeStudioStatus = `Applied ${preset} preset`;
        debouncedSave();
    }

    function applyAccessibilityPreset(preset: "focus" | "readability" | "balanced") {
        if (preset === "focus") {
            config.accessibility.reduced_motion = true;
            config.accessibility.text_scale = 1.05;
            config.accessibility.spacing_preset = "compact";
        } else if (preset === "readability") {
            config.accessibility.reduced_motion = true;
            config.accessibility.text_scale = 1.2;
            config.accessibility.spacing_preset = "relaxed";
        } else {
            config.accessibility.reduced_motion = false;
            config.accessibility.text_scale = 1.0;
            config.accessibility.spacing_preset = "comfortable";
        }

        debouncedSave();
    }

    function onTextScaleChange(value: number) {
        config.accessibility.text_scale = Math.max(0.85, Math.min(1.4, value));
        debouncedSave();
    }

    function exportThemeProfile() {
        const payload = {
            version: 1,
            appearance: config.appearance,
            window: config.window,
        };
        themeStudioText = JSON.stringify(payload, null, 2);
        themeStudioStatus = "Theme profile exported to editor";
    }

    async function copyThemeProfile() {
        try {
            await navigator.clipboard.writeText(themeStudioText || JSON.stringify({ version: 1, appearance: config.appearance, window: config.window }, null, 2));
            themeStudioStatus = "Theme profile copied";
        } catch {
            themeStudioStatus = "Copy failed (clipboard unavailable)";
        }
    }

    function importThemeProfile() {
        try {
            const parsed = JSON.parse(themeStudioText);
            if (!parsed?.appearance?.colors) {
                themeStudioStatus = "Invalid payload: missing appearance.colors";
                return;
            }

            const c = parsed.appearance.colors;
            const required = ["background", "surface", "accent", "accent_glow", "text_primary", "text_secondary", "border"];
            if (!required.every((k) => typeof c[k] === "string")) {
                themeStudioStatus = "Invalid payload: color keys must be strings";
                return;
            }

            config.appearance.colors = {
                background: c.background,
                surface: c.surface,
                accent: c.accent,
                accent_glow: c.accent_glow,
                text_primary: c.text_primary,
                text_secondary: c.text_secondary,
                border: c.border,
            };

            if (typeof parsed.appearance.blur_radius === "number") {
                config.appearance.blur_radius = Math.max(0, Math.min(100, Math.round(parsed.appearance.blur_radius)));
            }
            if (typeof parsed.appearance.opacity === "number") {
                config.appearance.opacity = Math.max(0.1, Math.min(1, parsed.appearance.opacity));
            }
            if (typeof parsed.appearance.border_radius === "number") {
                config.appearance.border_radius = Math.max(0, Math.min(50, Math.round(parsed.appearance.border_radius)));
            }
            if (typeof parsed.appearance.theme === "string") {
                config.appearance.theme = parsed.appearance.theme;
            }

            if (parsed.window && typeof parsed.window.width === "number" && typeof parsed.window.height === "number") {
                config.window.width = Math.max(400, Math.min(1920, parsed.window.width));
                config.window.height = Math.max(300, Math.min(1080, parsed.window.height));
                updateWindowSize();
            }

            themeStudioStatus = "Theme profile imported";
            debouncedSave();
        } catch {
            themeStudioStatus = "Invalid JSON payload";
        }
    }

    function resetThemeProfile() {
        config.appearance.blur_radius = DEFAULT_APPEARANCE.blur_radius;
        config.appearance.opacity = DEFAULT_APPEARANCE.opacity;
        config.appearance.border_radius = DEFAULT_APPEARANCE.border_radius;
        config.appearance.theme = DEFAULT_APPEARANCE.theme;
        config.appearance.colors = { ...DEFAULT_APPEARANCE.colors };
        themeStudioStatus = "Theme profile reset to defaults";
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

    function flushAndClose() {
        if (saveTimeout) clearTimeout(saveTimeout);
        invoke("save_config", { newConfig: config }).catch(console.error);
        onClose();
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === "Escape") {
            e.stopPropagation();
            flushAndClose();
        }
    }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="settings-panel">
    <header>
        <h2>Settings</h2>
        <div class="actions">
            <button class="close-btn" onclick={flushAndClose}>Done</button>
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

            <h3 style="margin-top: 1rem;">Theme Studio</h3>

            <div class="preset-row">
                <button class="preset-btn" onclick={() => applyThemePreset("minimal")}>Minimal</button>
                <button class="preset-btn" onclick={() => applyThemePreset("vivid")}>Vivid</button>
                <button class="preset-btn" onclick={() => applyThemePreset("high-contrast")}>High Contrast</button>
                <button class="preset-btn" onclick={() => applyThemePreset("compact")}>Compact</button>
            </div>

            <div class="control-group">
                <label>
                    Theme Profile JSON
                    <textarea
                        rows="8"
                        bind:value={themeStudioText}
                        placeholder="Export profile, edit values, then import"
                    ></textarea>
                </label>
            </div>

            <div class="preset-row">
                <button class="preset-btn" onclick={exportThemeProfile}>Export</button>
                <button class="preset-btn" onclick={copyThemeProfile}>Copy</button>
                <button class="preset-btn" onclick={importThemeProfile}>Import</button>
                <button class="preset-btn" onclick={resetThemeProfile}>Reset Default</button>
            </div>

            {#if themeStudioStatus}
                <div class="status-info">{themeStudioStatus}</div>
            {/if}
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
            <div class="control-group">
                <label>
                    Opt In To Popular Workflows Feed
                    <input
                        type="checkbox"
                        bind:checked={config.general.community_feed_opt_in}
                        onchange={onCommunityFeedOptInChange}
                    />
                </label>
            </div>
                </div>
            {/if}
        </div>

        <!-- Community Section -->
        <div class="accordion-item" class:active={activeSection === "Community"}>
            <button
                class="accordion-header"
                type="button"
                onclick={() => toggleSection("Community")}
                aria-expanded={activeSection === "Community"}
            >
                <svg class="accordion-icon" width="16" height="16" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>
                <h3>Community</h3>
            </button>
            {#if activeSection === "Community"}
                <div class="accordion-content">
                    <div class="control-group">
                        <label>
                            Feedback Message
                            <textarea
                                rows="5"
                                bind:value={feedbackMessage}
                                placeholder="Tell us what to improve in Vanta"
                            ></textarea>
                        </label>
                    </div>
                    <div class="control-group">
                        <label>
                            Contact (optional)
                            <input
                                type="text"
                                bind:value={feedbackContact}
                                placeholder="email or handle"
                            />
                        </label>
                    </div>
                    <div class="control-group">
                        <label>
                            Roadmap Topic
                            <select class="vanta-select" bind:value={selectedRoadmapTopic}>
                                {#each roadmapTopics as topic}
                                    <option value={topic.id}>{topic.label}</option>
                                {/each}
                            </select>
                        </label>
                    </div>
                    <div class="preset-row">
                        <button class="preset-btn" onclick={submitCommunityFeedback} disabled={communityBusy}>
                            {communityBusy ? "Submitting..." : "Submit Feedback"}
                        </button>
                        {#each roadmapTopics as topic}
                            <button
                                class="preset-btn"
                                disabled={communityBusy}
                                onclick={() => voteRoadmapTopic(topic.id)}
                            >
                                Vote: {topic.label}
                            </button>
                        {/each}
                    </div>

                    <div class="control-group" style="display: block;">
                        <h4>Popular Workflows Feed (Opt-in)</h4>
                        {#if !config.general.community_feed_opt_in}
                            <p>Enable opt-in under General to fetch curated popular workflows.</p>
                        {:else if communityFeed.length === 0}
                            <p>No popular workflows available right now.</p>
                        {:else}
                            <ul class="hint-list">
                                {#each communityFeed as item}
                                    <li>
                                        <strong>{item.title}</strong> ({item.id}) - {item.description}
                                        <br />
                                        <span>Tags: {item.tags.join(", ")} | Trust: {item.trust.publisher} / {item.trust.risk_level}</span>
                                        <br />
                                        <button class="preset-btn" disabled={communityBusy} onclick={() => installPopularWorkflow(item.id)}>
                                            Install Workflow
                                        </button>
                                    </li>
                                {/each}
                            </ul>
                        {/if}
                    </div>

                    <div class="control-group" style="display: block;">
                        <h4>Shareable Snippets</h4>
                        <div class="preset-row">
                            <button class="preset-btn" onclick={() => onSnippetKindChange("workflow")}>Workflow</button>
                            <button class="preset-btn" onclick={() => onSnippetKindChange("profile")}>Profile</button>
                            <button class="preset-btn" onclick={() => onSnippetKindChange("theme")}>Theme</button>
                        </div>

                        {#if snippetKind !== "theme"}
                            <label>
                                Target ID
                                <select class="vanta-select" bind:value={snippetTargetId}>
                                    {#if snippetKind === "workflow"}
                                        {#each availableWorkflowIds as id}
                                            <option value={id}>{id}</option>
                                        {/each}
                                    {:else}
                                        {#each availableProfileIds as id}
                                            <option value={id}>{id}</option>
                                        {/each}
                                    {/if}
                                </select>
                            </label>
                        {/if}

                        <div class="preset-row">
                            <button class="preset-btn" onclick={exportCommunitySnippet}>Export Snippet</button>
                            <button class="preset-btn" onclick={importCommunitySnippet}>Import Snippet</button>
                        </div>

                        <label>
                            Snippet Payload JSON
                            <textarea rows="8" bind:value={snippetPayload} placeholder="Paste or export snippet JSON"></textarea>
                        </label>

                        {#if snippetStatus}
                            <div class="status-info">{snippetStatus}</div>
                        {/if}
                    </div>

                    {#if communitySummary}
                        <div class="control-group" style="display: block;">
                            <h4>Community Snapshot</h4>
                            <p>Total feedback: {communitySummary.total_feedback}</p>
                            <ul class="hint-list">
                                {#each communitySummary.top_votes as vote}
                                    <li><strong>{vote.topic}</strong>: {vote.votes}</li>
                                {/each}
                            </ul>
                        </div>
                    {/if}

                    {#if feedbackStatus}
                        <div class="status-info">{feedbackStatus}</div>
                    {/if}
                </div>
            {/if}
        </div>

        <!-- Feature Hub Section -->
        <div class="accordion-item" class:active={activeSection === "Feature Hub"}>
            <button
                class="accordion-header"
                type="button"
                onclick={() => toggleSection("Feature Hub")}
                aria-expanded={activeSection === "Feature Hub"}
            >
                <svg class="accordion-icon" width="16" height="16" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>
                <h3>Feature Hub</h3>
            </button>
            {#if activeSection === "Feature Hub"}
                <div class="accordion-content">
                    <div class="control-group" style="display: block;">
                        <h4>Start Here</h4>
                        <p>Use these shortcuts to access major Vanta features quickly.</p>
                        <div class="preset-row">
                            <button class="preset-btn" onclick={() => jumpToSection("Theme Profile")}>Theme/Profile Studio</button>
                            <button class="preset-btn" onclick={() => jumpToSection("Community")}>Sharing & Feedback</button>
                            <button class="preset-btn" onclick={onOpenStore}>Extension Store</button>
                            <button class="preset-btn" onclick={() => jumpToSection("Search Ranking")}>Search & Workflows</button>
                        </div>
                    </div>

                    <div class="control-group" style="display: block;">
                        <h4>Extension Template Starter</h4>
                        <label>
                            Extension Name
                            <input type="text" bind:value={templateName} placeholder="my-extension" />
                        </label>
                        <div class="preset-row">
                            <button class="preset-btn" disabled={templateBusy} onclick={createExtensionTemplateFromHub}>
                                {templateBusy ? "Creating..." : "Create Template"}
                            </button>
                            <button class="preset-btn" onclick={() => jumpToSection("Community")}>Open Sharing Snippets</button>
                        </div>
                        {#if templateStatus}
                            <div class="status-info">{templateStatus}</div>
                        {/if}
                    </div>
                </div>
            {/if}
        </div>

        <!-- Accessibility Section -->
        <div class="accordion-item" class:active={activeSection === "Accessibility"}>
            <button
                class="accordion-header"
                type="button"
                onclick={() => toggleSection("Accessibility")}
                aria-expanded={activeSection === "Accessibility"}
            >
                <svg class="accordion-icon" width="16" height="16" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>
                <h3>Accessibility</h3>
            </button>
            {#if activeSection === "Accessibility"}
                <div class="accordion-content">
                    <div class="preset-row">
                        <button class="preset-btn" onclick={() => applyAccessibilityPreset("focus")}>Focus</button>
                        <button class="preset-btn" onclick={() => applyAccessibilityPreset("readability")}>Readability</button>
                        <button class="preset-btn" onclick={() => applyAccessibilityPreset("balanced")}>Balanced</button>
                    </div>

                    <div class="control-group">
                        <label>
                            Reduced Motion
                            <input
                                type="checkbox"
                                bind:checked={config.accessibility.reduced_motion}
                                onchange={debouncedSave}
                            />
                        </label>
                    </div>

                    <div class="control-group">
                        <label>
                            Text Scale ({config.accessibility.text_scale.toFixed(2)}x)
                            <input
                                type="range"
                                min="0.85"
                                max="1.40"
                                step="0.05"
                                value={config.accessibility.text_scale}
                                oninput={(e) => onTextScaleChange(Number((e.target as HTMLInputElement).value))}
                            />
                        </label>
                    </div>

                    <div class="control-group">
                        <label>
                            Spacing Preset
                            <select
                                class="vanta-select"
                                bind:value={config.accessibility.spacing_preset}
                                onchange={debouncedSave}
                            >
                                <option value="compact">Compact</option>
                                <option value="comfortable">Comfortable</option>
                                <option value="relaxed">Relaxed</option>
                            </select>
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
<div class="preset-row">
                <button class="preset-btn" onclick={() => applySearchPreset("developer")}>Developer</button>
                <button class="preset-btn" onclick={() => applySearchPreset("creator")}>Creator</button>
                <button class="preset-btn" onclick={() => applySearchPreset("minimal")}>Minimal</button>
            </div>
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

            <div class="control-group">
                <label>
                    Show "Why These Results"
                    <input
                        type="checkbox"
                        bind:checked={config.search.show_explain_panel}
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
                <button class="close-btn" onclick={loadHealthDashboard}>Refresh Health</button>
                <button class="close-btn" onclick={loadRecoveryHints}>Refresh Hints</button>
                <button class="close-btn" onclick={buildSupportBundle} disabled={supportBusy}>
                    {supportBusy ? "Building..." : "Create Support Bundle"}
                </button>
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

            {#if healthDashboard}
                <div class="control-group">
                    <label>
                        Active Profile
                        <input type="text" value={healthDashboard.active_profile_id} readonly />
                    </label>
                    <label>
                        Config Schema
                        <input type="text" value={`${healthDashboard.config_schema}`} readonly />
                    </label>
                    <label>
                        Indexed Entries
                        <input type="text" value={`${healthDashboard.file_index_entries}`} readonly />
                    </label>
                </div>
                <div class="control-group">
                    <label>
                        Apps Cached
                        <input type="text" value={`${healthDashboard.apps_cached}`} readonly />
                    </label>
                    <label>
                        Extensions Cached
                        <input type="text" value={`${healthDashboard.extensions_cached}`} readonly />
                    </label>
                    <label>
                        Macro Jobs
                        <input type="text" value={`${healthDashboard.macro_jobs_total}`} readonly />
                    </label>
                </div>
                <div class="control-group" style="display: block;">
                    <h4>Health Checks</h4>
                    <ul class="hint-list">
                        {#each healthDashboard.checks as check}
                            <li>
                                <strong>{check.name}</strong> [{check.status}] - {check.detail}
                            </li>
                        {/each}
                    </ul>
                </div>
            {/if}

            <div class="control-group" style="display: block;">
                <h4>Recovery Hints</h4>
                <ul class="hint-list">
                    {#each recoveryHints as hint}
                        <li>
                            <strong>{hint.title}</strong> - {hint.detail}
                        </li>
                    {/each}
                </ul>
            </div>

            {#if supportBundle}
                <div class="control-group" style="display: block;">
                    <label>
                        Latest Support Bundle
                        <input type="text" value={supportBundle.path} readonly />
                    </label>
                    <label>
                        Size (bytes)
                        <input type="text" value={`${supportBundle.size_bytes}`} readonly />
                    </label>
                </div>
            {/if}
                </div>
            {/if}
        </div>
    </div>
</div>

<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import type { AdaptiveAppearanceConfig, VantaConfig, ThemeMeta } from "$lib/types";

    let {
        config = $bindable(),
        availableThemes = [],
        onSave,
    }: {
        config: VantaConfig;
        availableThemes: ThemeMeta[];
        onSave: () => void;
    } = $props();

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
        adaptive: {
            enabled: false,
            profile: "balanced",
            lighting: "neutral",
            density: "comfortable",
            performance_tier: "balanced",
            accessibility_preset: "inherit",
        },
    } as const;

    const ADAPTIVE_PROFILE_OPTIONS = ["balanced", "daylight", "night", "presentation"] as const;
    const ADAPTIVE_LIGHTING_OPTIONS = ["neutral", "bright", "dim"] as const;
    const ADAPTIVE_DENSITY_OPTIONS = ["compact", "comfortable", "relaxed"] as const;
    const ADAPTIVE_PERFORMANCE_OPTIONS = ["quality", "balanced", "battery"] as const;
    const ADAPTIVE_ACCESSIBILITY_OPTIONS = ["inherit", "readability", "focus", "calm"] as const;

    function normalizeAdaptiveOption<T extends readonly string[]>(value: unknown, allowed: T, fallback: T[number]): T[number] {
        return typeof value === "string" && (allowed as readonly string[]).includes(value) ? (value as T[number]) : fallback;
    }

    function ensureAdaptiveProfile(): void {
        const adaptive = config.appearance.adaptive;
        const fallback = DEFAULT_APPEARANCE.adaptive;
        if (!adaptive || typeof adaptive !== "object") {
            config.appearance.adaptive = { ...fallback };
            return;
        }

        config.appearance.adaptive.enabled = Boolean(adaptive.enabled);
        config.appearance.adaptive.profile = normalizeAdaptiveOption(adaptive.profile, ADAPTIVE_PROFILE_OPTIONS, fallback.profile);
        config.appearance.adaptive.lighting = normalizeAdaptiveOption(adaptive.lighting, ADAPTIVE_LIGHTING_OPTIONS, fallback.lighting);
        config.appearance.adaptive.density = normalizeAdaptiveOption(adaptive.density, ADAPTIVE_DENSITY_OPTIONS, fallback.density);
        config.appearance.adaptive.performance_tier = normalizeAdaptiveOption(
            adaptive.performance_tier,
            ADAPTIVE_PERFORMANCE_OPTIONS,
            fallback.performance_tier,
        );
        config.appearance.adaptive.accessibility_preset = normalizeAdaptiveOption(
            adaptive.accessibility_preset,
            ADAPTIVE_ACCESSIBILITY_OPTIONS,
            fallback.accessibility_preset,
        );
    }

    function resetAdaptiveProfile() {
        config.appearance.adaptive = { ...DEFAULT_APPEARANCE.adaptive } as AdaptiveAppearanceConfig;
        themeStudioStatus = "Adaptive profile reset to defaults";
        onSave();
    }

    ensureAdaptiveProfile();

    function onThemeChange() {
        const selected = availableThemes.find((t) => t.id === config.appearance.theme);
        if (selected) {
            invoke("resize_window_for_theme", { width: selected.width, height: selected.height });
        }
        onSave();
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
            import("@tauri-apps/api/webviewWindow").then(({ getCurrentWebviewWindow }) => {
                import("@tauri-apps/api/dpi").then(({ LogicalSize }) => {
                    getCurrentWebviewWindow().setSize(new LogicalSize(config.window.width, config.window.height));
                });
            });
        }
        themeStudioStatus = `Applied ${preset} preset`;
        onSave();
    }

    function exportThemeProfile() {
        const payload = { version: 1, appearance: config.appearance, window: config.window };
        themeStudioText = JSON.stringify(payload, null, 2);
        themeStudioStatus = "Theme profile exported to editor";
    }

    async function copyThemeProfile() {
        try {
            await navigator.clipboard.writeText(
                themeStudioText || JSON.stringify({ version: 1, appearance: config.appearance, window: config.window }, null, 2),
            );
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
                background: c.background, surface: c.surface, accent: c.accent,
                accent_glow: c.accent_glow, text_primary: c.text_primary,
                text_secondary: c.text_secondary, border: c.border,
            };
            if (typeof parsed.appearance.blur_radius === "number")
                config.appearance.blur_radius = Math.max(0, Math.min(100, Math.round(parsed.appearance.blur_radius)));
            if (typeof parsed.appearance.opacity === "number")
                config.appearance.opacity = Math.max(0.1, Math.min(1, parsed.appearance.opacity));
            if (typeof parsed.appearance.border_radius === "number")
                config.appearance.border_radius = Math.max(0, Math.min(50, Math.round(parsed.appearance.border_radius)));
            if (typeof parsed.appearance.theme === "string")
                config.appearance.theme = parsed.appearance.theme;
            if (parsed.appearance.adaptive && typeof parsed.appearance.adaptive === "object") {
                const adaptive = parsed.appearance.adaptive;
                config.appearance.adaptive = {
                    enabled: Boolean(adaptive.enabled),
                    profile: normalizeAdaptiveOption(adaptive.profile, ADAPTIVE_PROFILE_OPTIONS, DEFAULT_APPEARANCE.adaptive.profile),
                    lighting: normalizeAdaptiveOption(adaptive.lighting, ADAPTIVE_LIGHTING_OPTIONS, DEFAULT_APPEARANCE.adaptive.lighting),
                    density: normalizeAdaptiveOption(adaptive.density, ADAPTIVE_DENSITY_OPTIONS, DEFAULT_APPEARANCE.adaptive.density),
                    performance_tier: normalizeAdaptiveOption(
                        adaptive.performance_tier,
                        ADAPTIVE_PERFORMANCE_OPTIONS,
                        DEFAULT_APPEARANCE.adaptive.performance_tier,
                    ),
                    accessibility_preset: normalizeAdaptiveOption(
                        adaptive.accessibility_preset,
                        ADAPTIVE_ACCESSIBILITY_OPTIONS,
                        DEFAULT_APPEARANCE.adaptive.accessibility_preset,
                    ),
                };
            }
            if (parsed.window && typeof parsed.window.width === "number" && typeof parsed.window.height === "number") {
                config.window.width = Math.max(400, Math.min(1920, parsed.window.width));
                config.window.height = Math.max(300, Math.min(1080, parsed.window.height));
                import("@tauri-apps/api/webviewWindow").then(({ getCurrentWebviewWindow }) => {
                    import("@tauri-apps/api/dpi").then(({ LogicalSize }) => {
                        getCurrentWebviewWindow().setSize(new LogicalSize(config.window.width, config.window.height));
                    });
                });
            }
            themeStudioStatus = "Theme profile imported";
            onSave();
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
        config.appearance.adaptive = { ...DEFAULT_APPEARANCE.adaptive } as AdaptiveAppearanceConfig;
        themeStudioStatus = "Theme profile reset to defaults";
        onSave();
    }
</script>

<div class="control-group">
    <label>
        Active Theme
        <select class="vanta-select" bind:value={config.appearance.theme} onchange={onThemeChange}>
            {#each availableThemes as theme}
                <option value={theme.id}>{theme.name}</option>
            {/each}
        </select>
    </label>
</div>

<h3 class="settings-subheading">Colors (Overrides)</h3>

<div class="control-group">
    <label>Background Color <input type="color" bind:value={config.appearance.colors.background} oninput={onSave} /></label>
    <label>Surface Color <input type="color" bind:value={config.appearance.colors.surface} oninput={onSave} /></label>
    <label>Accent Color <input type="color" bind:value={config.appearance.colors.accent} oninput={onSave} /></label>
    <label>Border Color <input type="text" bind:value={config.appearance.colors.border} oninput={onSave} /></label>
</div>

<div class="control-group">
    <label>Opacity ({config.appearance.opacity})
        <input type="range" min="0.1" max="1" step="0.05" bind:value={config.appearance.opacity} oninput={onSave} />
    </label>
</div>

<div class="control-group">
    <label>Border Radius ({config.appearance.border_radius}px)
        <input type="range" min="0" max="50" step="1" bind:value={config.appearance.border_radius} oninput={onSave} />
    </label>
</div>

<div class="control-group">
    <label>Blur Radius ({config.appearance.blur_radius}px)
        <input type="range" min="0" max="100" step="1" bind:value={config.appearance.blur_radius} oninput={onSave} />
    </label>
</div>

<h3 class="settings-subheading">Adaptive Appearance</h3>

<div class="control-group">
    <label>
        <input type="checkbox" bind:checked={config.appearance.adaptive.enabled} onchange={onSave} />
        Enable adaptive profile transforms
    </label>
</div>

<div class="control-group">
    <label>
        Profile
        <select class="vanta-select" bind:value={config.appearance.adaptive.profile} onchange={onSave}>
            {#each ADAPTIVE_PROFILE_OPTIONS as profile}
                <option value={profile}>{profile}</option>
            {/each}
        </select>
    </label>
    <label>
        Lighting
        <select class="vanta-select" bind:value={config.appearance.adaptive.lighting} onchange={onSave}>
            {#each ADAPTIVE_LIGHTING_OPTIONS as lighting}
                <option value={lighting}>{lighting}</option>
            {/each}
        </select>
    </label>
    <label>
        Density
        <select class="vanta-select" bind:value={config.appearance.adaptive.density} onchange={onSave}>
            {#each ADAPTIVE_DENSITY_OPTIONS as density}
                <option value={density}>{density}</option>
            {/each}
        </select>
    </label>
    <label>
        Performance Tier
        <select class="vanta-select" bind:value={config.appearance.adaptive.performance_tier} onchange={onSave}>
            {#each ADAPTIVE_PERFORMANCE_OPTIONS as tier}
                <option value={tier}>{tier}</option>
            {/each}
        </select>
    </label>
    <label>
        Accessibility Preset
        <select class="vanta-select" bind:value={config.appearance.adaptive.accessibility_preset} onchange={onSave}>
            {#each ADAPTIVE_ACCESSIBILITY_OPTIONS as preset}
                <option value={preset}>{preset}</option>
            {/each}
        </select>
    </label>
</div>

<div class="preset-row">
    <button class="preset-btn" onclick={resetAdaptiveProfile}>Reset Adaptive Profile</button>
</div>

<h3 class="settings-subheading">Theme Studio</h3>

<div class="preset-row">
    <button class="preset-btn" onclick={() => applyThemePreset("minimal")}>Minimal</button>
    <button class="preset-btn" onclick={() => applyThemePreset("vivid")}>Vivid</button>
    <button class="preset-btn" onclick={() => applyThemePreset("high-contrast")}>High Contrast</button>
    <button class="preset-btn" onclick={() => applyThemePreset("compact")}>Compact</button>
</div>

<div class="control-group">
    <label>Theme Profile JSON
        <textarea rows="8" bind:value={themeStudioText} placeholder="Export profile, edit values, then import"></textarea>
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

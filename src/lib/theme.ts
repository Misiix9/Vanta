import type { VantaConfig, ThemeDiagnostic } from "./types";

// ── Phase 30 · Theme Token Contract ──────────────────────────

/** Tokens every theme MUST define for safe rendering. */
export const REQUIRED_TOKENS: string[] = [
    "--vanta-width",
    "--vanta-height",
    "--bg",
    "--surface",
    "--text-primary",
    "--text-secondary",
    "--purple",
    "--ds-accent",
    "--ds-surface-0",
    "--ds-surface-1",
    "--ds-border",
    "--ds-text-primary",
    "--ds-text-secondary",
    "--font-ui",
    "--radius",
];

/** Tokens themes SHOULD define for full visual fidelity. */
export const RECOMMENDED_TOKENS: string[] = [
    "--ds-surface-2",
    "--ds-surface-3",
    "--ds-accent-glow",
    "--ds-text-muted",
    "--ds-success",
    "--ds-warning",
    "--ds-danger",
    "--ds-info",
    "--ds-neutral",
    "--font-mono",
    "--type-display",
    "--type-heading",
    "--type-body",
    "--type-caption",
    "--space-1",
    "--space-2",
    "--space-3",
    "--radius-item",
    "--motion-enter-duration",
    "--motion-ease-enter",
    "--icon-sm",
    "--icon-md",
    "--icon-lg",
    "--icon-xl",
    "--border-window",
];

const TOKEN_DEF_RE = /--([\w-]+)\s*:/g;
const DIMENSION_RE = /--vanta-(width|height)\s*:\s*(\d+(?:\.\d+)?)\s*px/g;

const ADAPTIVE_DEFAULTS = {
    enabled: false,
    profile: "balanced",
    lighting: "neutral",
    density: "comfortable",
    performance_tier: "balanced",
    accessibility_preset: "inherit",
} as const;

const ADAPTIVE_PROFILE_OPTIONS = ["balanced", "daylight", "night", "presentation"] as const;
const ADAPTIVE_LIGHTING_OPTIONS = ["neutral", "bright", "dim"] as const;
const ADAPTIVE_DENSITY_OPTIONS = ["compact", "comfortable", "relaxed"] as const;
const ADAPTIVE_PERFORMANCE_OPTIONS = ["quality", "balanced", "battery"] as const;
const ADAPTIVE_ACCESSIBILITY_OPTIONS = ["inherit", "readability", "focus", "calm"] as const;

type AdaptiveSettings = {
    enabled: boolean;
    profile: (typeof ADAPTIVE_PROFILE_OPTIONS)[number];
    lighting: (typeof ADAPTIVE_LIGHTING_OPTIONS)[number];
    density: (typeof ADAPTIVE_DENSITY_OPTIONS)[number];
    performance_tier: (typeof ADAPTIVE_PERFORMANCE_OPTIONS)[number];
    accessibility_preset: (typeof ADAPTIVE_ACCESSIBILITY_OPTIONS)[number];
};

type AdaptedAppearance = {
    blur: number;
    opacity: number;
    radius: number;
    textScale: number;
    spacingScale: number;
    motionScale: number;
    settings: AdaptiveSettings;
};

function clamp(value: number, min: number, max: number): number {
    return Math.max(min, Math.min(max, value));
}

function round(value: number, digits = 3): number {
    const factor = 10 ** digits;
    return Math.round(value * factor) / factor;
}

function normalizeOption<T extends readonly string[]>(
    value: string | undefined,
    options: T,
    fallback: T[number],
): T[number] {
    if (typeof value !== "string") {
        return fallback;
    }
    return (options as readonly string[]).includes(value) ? (value as T[number]) : fallback;
}

function resolveSpacingScale(spacingPreset: string | undefined): number {
    if (spacingPreset === "compact") {
        return 0.88;
    }
    if (spacingPreset === "relaxed") {
        return 1.16;
    }
    return 1;
}

function resolveAdaptiveSettings(config: VantaConfig): AdaptiveSettings {
    const incoming = config.appearance.adaptive;

    return {
        enabled: Boolean(incoming?.enabled),
        profile: normalizeOption(incoming?.profile, ADAPTIVE_PROFILE_OPTIONS, ADAPTIVE_DEFAULTS.profile),
        lighting: normalizeOption(incoming?.lighting, ADAPTIVE_LIGHTING_OPTIONS, ADAPTIVE_DEFAULTS.lighting),
        density: normalizeOption(incoming?.density, ADAPTIVE_DENSITY_OPTIONS, ADAPTIVE_DEFAULTS.density),
        performance_tier: normalizeOption(
            incoming?.performance_tier,
            ADAPTIVE_PERFORMANCE_OPTIONS,
            ADAPTIVE_DEFAULTS.performance_tier,
        ),
        accessibility_preset: normalizeOption(
            incoming?.accessibility_preset,
            ADAPTIVE_ACCESSIBILITY_OPTIONS,
            ADAPTIVE_DEFAULTS.accessibility_preset,
        ),
    };
}

function resolveAdaptedAppearance(config: VantaConfig): AdaptedAppearance {
    const settings = resolveAdaptiveSettings(config);
    const baseTextScale = clamp(config.accessibility.text_scale || 1, 0.85, 1.4);
    const baseSpacingScale = resolveSpacingScale(config.accessibility.spacing_preset);
    const reducedMotion = Boolean(config.accessibility.reduced_motion);

    let blur = config.appearance.blur_radius;
    let opacity = config.appearance.opacity;
    let radius = config.appearance.border_radius;
    let textScale = baseTextScale;
    let spacingScale = baseSpacingScale;
    let motionScale = reducedMotion ? 0 : 1;

    if (settings.enabled) {
        if (settings.profile === "daylight") {
            blur -= 6;
            opacity += 0.03;
            radius += 1;
        } else if (settings.profile === "night") {
            blur += 8;
            opacity -= 0.03;
            radius += 2;
        } else if (settings.profile === "presentation") {
            blur -= 10;
            opacity += 0.06;
            radius -= 2;
            textScale += 0.04;
            spacingScale += 0.05;
        }

        if (settings.lighting === "bright") {
            blur -= 8;
            opacity += 0.04;
        } else if (settings.lighting === "dim") {
            blur += 6;
            opacity -= 0.04;
        }

        if (settings.density === "compact") {
            spacingScale *= 0.92;
            textScale *= 0.97;
            radius -= 2;
        } else if (settings.density === "relaxed") {
            spacingScale *= 1.1;
            textScale *= 1.03;
            radius += 2;
        }

        if (settings.performance_tier === "quality") {
            blur *= 1.08;
        } else if (settings.performance_tier === "battery") {
            blur *= 0.55;
            motionScale *= 0.75;
            opacity += 0.02;
        }

        if (settings.accessibility_preset === "readability") {
            textScale *= 1.08;
            spacingScale *= 1.08;
            opacity += 0.03;
        } else if (settings.accessibility_preset === "focus") {
            blur *= 0.75;
            opacity += 0.04;
            motionScale *= 0.75;
        } else if (settings.accessibility_preset === "calm") {
            blur *= 0.85;
            motionScale = 0;
        }
    }

    if (reducedMotion) {
        motionScale = 0;
    }

    return {
        blur: Math.round(clamp(blur, 0, 100)),
        opacity: round(clamp(opacity, 0.1, 1), 3),
        radius: Math.round(clamp(radius, 0, 50)),
        textScale: round(clamp(textScale, 0.85, 1.4), 3),
        spacingScale: round(clamp(spacingScale, 0.8, 1.25), 3),
        motionScale: round(clamp(motionScale, 0, 1), 3),
        settings,
    };
}

/**
 * Validate a theme CSS string against the token contract.
 * Returns an array of diagnostics (empty = healthy theme).
 */
export function validateThemeTokens(cssContent: string): ThemeDiagnostic[] {
    const diagnostics: ThemeDiagnostic[] = [];
    const defined = new Set<string>();

    let m: RegExpExecArray | null;
    TOKEN_DEF_RE.lastIndex = 0;
    while ((m = TOKEN_DEF_RE.exec(cssContent)) !== null) {
        defined.add("--" + m[1]);
    }

    for (const token of REQUIRED_TOKENS) {
        if (!defined.has(token)) {
            diagnostics.push({
                level: "error",
                token,
                message: `Required token ${token} is missing. The UI may not render correctly.`,
            });
        }
    }

    for (const token of RECOMMENDED_TOKENS) {
        if (!defined.has(token)) {
            diagnostics.push({
                level: "warning",
                token,
                message: `Recommended token ${token} is missing. Some visual details may fall back to defaults.`,
            });
        }
    }

    // Validate dimension ranges
    DIMENSION_RE.lastIndex = 0;
    while ((m = DIMENSION_RE.exec(cssContent)) !== null) {
        const dim = m[1]; // "width" or "height"
        const val = parseFloat(m[2]);
        if (val < 400 || val > 1200) {
            diagnostics.push({
                level: "warning",
                token: `--vanta-${dim}`,
                message: `--vanta-${dim} is ${val}px (expected 400–1200px). Window may be unusable.`,
            });
        }
    }

    return diagnostics;
}

// Applies theme CSS variables to :root.
export function applyTheme(config: VantaConfig): void {
    const { colors } = config.appearance;
    const { reduced_motion } = config.accessibility;
    const adapted = resolveAdaptedAppearance(config);
    const root = document.documentElement;
    const surfaceElevated = colors.surface_elevated ?? "color-mix(in srgb, " + colors.surface + " 78%, white 22%)";
    const success = colors.success ?? "#2dd4bf";
    const warning = colors.warning ?? "#f59e0b";
    const danger = colors.danger ?? "#ef4444";
    const info = colors.info ?? colors.accent;

    // Legacy color variables
    root.style.setProperty("--vanta-bg", colors.background);
    root.style.setProperty("--vanta-surface", colors.surface);
    root.style.setProperty("--vanta-accent", colors.accent);
    root.style.setProperty("--vanta-accent-glow", colors.accent_glow);
    root.style.setProperty("--vanta-text", colors.text_primary);
    root.style.setProperty("--vanta-text-dim", colors.text_secondary);
    root.style.setProperty("--vanta-border", colors.border);

    // Design-system semantic tokens
    root.style.setProperty("--ds-surface-0", colors.background);
    root.style.setProperty("--ds-surface-1", colors.surface);
    root.style.setProperty("--ds-surface-2", surfaceElevated);
    root.style.setProperty("--ds-surface-3", "color-mix(in srgb, " + surfaceElevated + " 78%, white 22%)");
    root.style.setProperty("--ds-accent", colors.accent);
    root.style.setProperty("--ds-accent-glow", colors.accent_glow);
    root.style.setProperty("--ds-text-primary", colors.text_primary);
    root.style.setProperty("--ds-text-secondary", colors.text_secondary);
    root.style.setProperty("--ds-text-muted", "color-mix(in srgb, " + colors.text_secondary + " 70%, black 30%)");
    root.style.setProperty("--ds-border", colors.border);
    root.style.setProperty("--ds-success", success);
    root.style.setProperty("--ds-warning", warning);
    root.style.setProperty("--ds-danger", danger);
    root.style.setProperty("--ds-info", info);

    // Back-compat aliases used by existing theme CSS
    root.style.setProperty("--bg", colors.background);
    root.style.setProperty("--surface", colors.surface);
    root.style.setProperty("--text-primary", colors.text_primary);
    root.style.setProperty("--text-secondary", colors.text_secondary);
    root.style.setProperty("--border-window", colors.border);

    // Appearance variables
    root.style.setProperty("--vanta-blur", `${adapted.blur}px`);
    root.style.setProperty("--vanta-opacity", `${adapted.opacity}`);
    root.style.setProperty("--vanta-radius", `${adapted.radius}px`);
    root.style.setProperty("--vanta-text-scale", `${adapted.textScale}`);
    root.style.setProperty("--vanta-space-scale", `${adapted.spacingScale}`);
    root.style.setProperty("--vanta-motion-scale", `${adapted.motionScale}`);
    root.style.setProperty("--vanta-adaptive-enabled", adapted.settings.enabled ? "1" : "0");

    root.dataset.appearanceProfile = adapted.settings.profile;
    root.dataset.appearanceLighting = adapted.settings.lighting;
    root.dataset.appearanceDensity = adapted.settings.density;
    root.dataset.appearancePerformance = adapted.settings.performance_tier;
    root.dataset.appearanceAccessibility = adapted.settings.accessibility_preset;
    root.dataset.reducedMotion = reduced_motion ? "true" : "false";
}

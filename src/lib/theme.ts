import type { VantaConfig, VantaColors, AppearanceConfig, ThemeDiagnostic } from "./types";

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
    const { colors, blur_radius, opacity, border_radius } = config.appearance;
    const { reduced_motion, text_scale, spacing_preset } = config.accessibility;
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
    root.style.setProperty("--vanta-blur", `${blur_radius}px`);
    root.style.setProperty("--vanta-opacity", `${opacity}`);
    root.style.setProperty("--vanta-radius", `${border_radius}px`);

    const clampedTextScale = Math.max(0.85, Math.min(1.4, text_scale || 1));
    root.style.setProperty("--vanta-text-scale", `${clampedTextScale}`);

    const spacingScale =
        spacing_preset === "compact"
            ? 0.88
            : spacing_preset === "relaxed"
              ? 1.16
              : 1;
    root.style.setProperty("--vanta-space-scale", `${spacingScale}`);
    root.style.setProperty("--vanta-motion-scale", reduced_motion ? "0" : "1");
    root.dataset.reducedMotion = reduced_motion ? "true" : "false";
}

import type { VantaConfig, VantaColors, AppearanceConfig } from "./types";

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

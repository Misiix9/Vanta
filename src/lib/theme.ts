import type { VantaConfig, VantaColors, AppearanceConfig } from "./types";

// Applies theme CSS variables to :root.
export function applyTheme(config: VantaConfig): void {
    const { colors, blur_radius, opacity, border_radius } = config.appearance;
    const root = document.documentElement;

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
    root.style.setProperty("--ds-surface-2", colors.surface);
    root.style.setProperty("--ds-accent", colors.accent);
    root.style.setProperty("--ds-accent-glow", colors.accent_glow);
    root.style.setProperty("--ds-text-primary", colors.text_primary);
    root.style.setProperty("--ds-text-secondary", colors.text_secondary);
    root.style.setProperty("--ds-border", colors.border);

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
}

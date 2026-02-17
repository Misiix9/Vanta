import type { VantaConfig, VantaColors, AppearanceConfig } from "./types";

// Applies theme CSS variables to :root.
export function applyTheme(config: VantaConfig): void {
    const { colors, blur_radius, opacity, border_radius } = config.appearance;
    const root = document.documentElement;

    // Color variables
    root.style.setProperty("--vanta-bg", colors.background);
    root.style.setProperty("--vanta-surface", colors.surface);
    root.style.setProperty("--vanta-accent", colors.accent);
    root.style.setProperty("--vanta-accent-glow", colors.accent_glow);
    root.style.setProperty("--vanta-text", colors.text_primary);
    root.style.setProperty("--vanta-text-dim", colors.text_secondary);
    root.style.setProperty("--vanta-border", colors.border);

    // Appearance variables
    root.style.setProperty("--vanta-blur", `${blur_radius}px`);
    root.style.setProperty("--vanta-opacity", `${opacity}`);
    root.style.setProperty("--vanta-radius", `${border_radius}px`);
}

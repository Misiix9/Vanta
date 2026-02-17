export interface VantaColors {
    background: string;
    surface: string;
    accent: string;
    accent_glow: string;
    text_primary: string;
    text_secondary: string;
    border: string;
}

export interface AppearanceConfig {
    blur_radius: number;
    opacity: number;
    border_radius: number;
    colors: VantaColors;
}

export interface GeneralConfig {
    hotkey: string;
    max_results: number;
    launch_on_login: boolean;
}

export interface ScriptsConfig {
    directory: string;
    timeout_ms: number;
}

export interface VantaConfig {
    general: GeneralConfig;
    appearance: AppearanceConfig;
    scripts: ScriptsConfig;
}

export interface SearchResult {
    title: string;
    subtitle: string | null;
    icon: string | null;
    exec: string;
    score: number;
    match_indices: number[];
    source: ResultSource;
}

export type ResultSource = "Application" | { Script: { keyword: string } };

export interface BlurStatus {
    mode: "native" | "fallback";
}



export interface ScriptAction {
    type: "copy" | "open" | "run";
    value: string;
}

export type Urgency = "low" | "normal" | "critical";

export interface ScriptItem {
    title: string;
    subtitle?: string;
    icon?: string;
    action?: ScriptAction;
    badge?: string;
    urgency: Urgency;
}

export interface ScriptOutput {
    items: ScriptItem[];
}

export interface ScriptEntry {
    keyword: string;
    name?: string;
    description?: string;
    icon?: string;
    path: string;
}

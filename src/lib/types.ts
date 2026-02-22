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
    theme: string;
    colors: VantaColors;
}

export interface ThemeMeta {
    id: string;
    name: string;
    width: number;
    height: number;
    css_content: string;
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

export interface WindowConfig {
    width: number;
    height: number;
}

export interface FilesConfig {
    include_hidden: boolean;
    max_depth: number;
    file_manager: string;
    file_editor: string;
    open_docs_in_manager: boolean;
}

export interface SourcePreference {
    enabled: boolean;
    weight: number;
}

export interface SearchConfig {
    applications: SourcePreference;
    windows: SourcePreference;
    calculator: SourcePreference;
    files: SourcePreference;
}

export interface VantaConfig {
    general: GeneralConfig;
    appearance: AppearanceConfig;
    window: WindowConfig;
    scripts: ScriptsConfig;
    files: FilesConfig;
    search: SearchConfig;
}

export interface PerfStats {
    calls: number;
    total_ms: number;
    avg_ms: number;
    max_ms: number;
}

export interface SearchDiagnostics {
    search: PerfStats;
    suggestions: PerfStats;
    launch: PerfStats;
}

export interface ResultAction {
    label: string;
    exec: string;
    shortcut?: string;
}

export interface SearchResult {
    title: string;
    subtitle: string | null;
    icon: string | null;
    exec: string;
    score: number;
    match_indices: number[];
    source: ResultSource;
    id?: number | string;
    actions?: ResultAction[];
}

export type ResultSource = "Application" | "Calculator" | "Window" | "Clipboard" | "File" | { Script: { keyword: string } };

export interface ClipboardItem {
    id: number;
    content: string;
    timestamp: string;
}

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

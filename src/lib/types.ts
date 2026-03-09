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

export interface ExtensionsConfig {
    directory: string;
    dev_mode: boolean;
}

export interface WindowConfig {
    width: number;
    height: number;
}

export interface AccessibilityConfig {
    reduced_motion: boolean;
    text_scale: number;
    spacing_preset: "compact" | "comfortable" | "relaxed" | string;
}

export interface FilesConfig {
    include_hidden: boolean;
    max_depth: number;
    file_manager: string;
    file_editor: string;
    open_docs_in_manager: boolean;
    include_globs: string[];
    exclude_globs: string[];
    allowed_extensions: string[];
    type_filter: string;
    indexed_at?: number | null;
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
    windows_max_results?: number;
}

export interface MacroArg {
    name: string;
    description?: string | null;
    required?: boolean;
    default_value?: string | null;
}

export type MacroStep =
    | { kind: "extension"; ext_id: string; command: string; args?: string[]; capabilities?: Capability[] }
    | { kind: "system"; command: string; args?: string[]; capabilities?: Capability[] };

export interface WorkflowMacro {
    id: string;
    name: string;
    description?: string | null;
    enabled?: boolean;
    args?: MacroArg[];
    steps: MacroStep[];
}

export interface WorkflowsConfig {
    macros?: WorkflowMacro[];
}

export interface MacroDryRunStep {
    index: number;
    kind: string;
    command: string;
    args: string[];
    capabilities: Capability[];
    decision: PermissionDecision;
    missing_caps: Capability[];
}

export interface MacroDryRunResult {
    macro_id: string;
    ready: boolean;
    errors: string[];
    steps: MacroDryRunStep[];
}

export interface MacroRunStepResult {
    index: number;
    kind: string;
    command: string;
    args: string[];
    capabilities: Capability[];
    status: string;
}

export interface MacroRunResult {
    macro_id: string;
    steps: MacroRunStepResult[];
}

export interface VantaConfig {
    general: GeneralConfig;
    appearance: AppearanceConfig;
    window: WindowConfig;
    accessibility: AccessibilityConfig;
    extensions: ExtensionsConfig;
    files: FilesConfig;
    search: SearchConfig;
    workflows: WorkflowsConfig;
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
    group?: string;
    section?: string;
}

export type ResultSource =
    | "Application"
    | "Calculator"
    | "Window"
    | "File"
    | "Clipboard"
    | { Extension: { ext_id: string } };

export interface ClipboardItem {
    id: number;
    content: string;
    timestamp: string;
    pinned: boolean;
    content_type: string;
}

export interface BlurStatus {
    mode: "native" | "fallback";
}

export type Capability = "Network" | "Shell" | "Filesystem";

export type PermissionDecision = "Allow" | "Deny" | "Ask";

export interface PermissionNeededPayload {
    script_id: string;
    missing_caps: Capability[];
    requested_caps?: Capability[];
}

export interface ExtensionCommand {
    name: string;
    title: string;
    subtitle?: string;
    mode: "view" | "no-view";
    icon?: string;
}

export interface ExtensionManifest {
    name: string;
    title: string;
    version: string;
    description?: string;
    author?: string;
    icon?: string;
    permissions: Capability[];
    commands: ExtensionCommand[];
}

export interface ExtensionEntry {
    manifest: ExtensionManifest;
    path: string;
    has_bundle: boolean;
    has_styles: boolean;
}

export interface StoreExtensionInfo {
    name: string;
    title: string;
    version: string;
    description: string;
    author: string;
    icon: string | null;
    permissions: string[];
    category: string;
    rating?: number | null;
    install_count: number;
    trust_badge: string;
    changelog: string[];
    permission_risk: "Low" | "Medium" | "High" | string;
    commands_count: number;
    installed: boolean;
}

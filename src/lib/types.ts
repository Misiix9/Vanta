export interface VantaColors {
    background: string;
    surface: string;
    accent: string;
    accent_glow: string;
    text_primary: string;
    text_secondary: string;
    border: string;
    surface_elevated?: string;
    success?: string;
    warning?: string;
    danger?: string;
    info?: string;
}

export interface AppearanceConfig {
    blur_radius: number;
    opacity: number;
    border_radius: number;
    theme: string;
    colors: VantaColors;
}

export interface ThemeDiagnostic {
    level: "error" | "warning" | "info";
    token: string;
    message: string;
}

export interface ThemeMeta {
    id: string;
    name: string;
    width: number;
    height: number;
    css_content: string;
    diagnostics: ThemeDiagnostic[];
}

export interface GeneralConfig {
    hotkey: string;
    max_results: number;
    launch_on_login: boolean;
    community_feed_opt_in?: boolean;
}

export interface CommunityVoteCount {
    topic: string;
    votes: number;
}

export interface CommunityTrustMetadata {
    source: string;
    publisher: string;
    verified: boolean;
    risk_level: string;
    signature_hint?: string | null;
}

export interface PopularWorkflowFeedEntry {
    id: string;
    title: string;
    description: string;
    tags: string[];
    trust: CommunityTrustMetadata;
    workflow: WorkflowMacro;
}

export interface CommunityFeedbackSummary {
    schema_version: number;
    updated_at: number;
    total_feedback: number;
    top_votes: CommunityVoteCount[];
}

export interface CommunityFeedbackReceipt {
    id: string;
    created_at: number;
    total_feedback: number;
}

export interface ExtensionsConfig {
    directory: string;
    dev_mode: boolean;
}

export interface PolicyConfig {
    restricted_mode: boolean;
    allowed_extensions: string[];
    blocked_capabilities: Capability[];
    require_verified_extensions: boolean;
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
    show_explain_panel: boolean;
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
    schema_version?: number;
    macros?: WorkflowMacro[];
}

export interface ProfileConfig {
    id: string;
    name: string;
    hotkey: string;
    theme: string;
    search: SearchConfig;
}

export interface ProfilesConfig {
    schema_version?: number;
    active_profile_id: string;
    entries: ProfileConfig[];
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

export type MacroJobStatus = "running" | "succeeded" | "failed" | "canceled";

export interface MacroJobRecord {
    id: string;
    macro_id: string;
    macro_name: string;
    args: Record<string, string>;
    status: MacroJobStatus;
    created_at: number;
    updated_at: number;
    completed_at?: number | null;
    steps: MacroRunStepResult[];
    error?: string | null;
}

export interface VantaConfig {
    schema_version?: number;
    general: GeneralConfig;
    appearance: AppearanceConfig;
    window: WindowConfig;
    accessibility: AccessibilityConfig;
    extensions: ExtensionsConfig;
    files: FilesConfig;
    search: SearchConfig;
    workflows: WorkflowsConfig;
    profiles?: ProfilesConfig;
    policy?: PolicyConfig;
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

export interface HealthCheck {
    name: string;
    status: string;
    detail: string;
}

export interface HealthDashboard {
    generated_at: number;
    config_schema: number;
    active_profile_id: string;
    apps_cached: number;
    extensions_cached: number;
    file_index_entries: number;
    macro_jobs_total: number;
    checks: HealthCheck[];
}

export interface SupportBundleReport {
    path: string;
    size_bytes: number;
}

export interface RecoveryHint {
    id: string;
    title: string;
    detail: string;
}

export interface ResultAction {
    label: string;
    exec?: string;
    shortcut?: string;
    command?: CommandContract;
}

export type CommandContract =
    | { kind: "launch_app"; exec: string }
    | { kind: "open_file"; path: string }
    | { kind: "open_settings" }
    | { kind: "open_settings_section"; section: string }
    | { kind: "open_feature_window"; window: string }
    | { kind: "open_store" }
    | { kind: "copy_text"; value: string }
    | { kind: "copy_path"; value: string }
    | { kind: "reveal_path"; path: string }
    | { kind: "open_with_editor"; path: string }
    | { kind: "focus_window"; id: string }
    | { kind: "close_window"; id: string }
    | { kind: "minimize_window"; id: string }
    | { kind: "move_window_current_workspace"; id: string }
    | { kind: "system_action"; action: string }
    | { kind: "macro_open"; id: string }
    | { kind: "extension_view"; ext_id: string; command: string }
    | { kind: "extension_action"; ext_id: string; command: string }
    | { kind: "query_fill"; value: string }
    | { kind: "intent_workflow"; steps: string[] }
    | { kind: "profile_switch"; id: string }
    | { kind: "unknown"; exec: string };

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
    version?: number;
    command?: CommandContract;
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

export interface ExtensionDependency {
    ext_id: string;
    version_range: string;
}

export interface ExtensionManifest {
    name: string;
    title: string;
    version: string;
    description?: string;
    author?: string;
    publisher?: string;
    safe?: boolean;
    icon?: string;
    permissions: Capability[];
    requires?: ExtensionDependency[];
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
    publisher: string;
    safe: boolean;
    icon: string | null;
    permissions: string[];
    category: string;
    rating?: number | null;
    rating_count: number;
    install_count: number;
    trust_badge: string;
    changelog: string[];
    permission_risk: "Low" | "Medium" | "High" | string;
    commands_count: number;
    installed: boolean;
}

export interface ExtensionUpdateInfo {
    name: string;
    current_version: string;
    latest_version: string;
}

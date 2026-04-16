<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import type { SearchDiagnostics, HealthDashboard, RecoveryHint, SupportBundleReport, ConfigAuditEntry, ConfigSchemaValidationReport, UsageAnalyticsReport } from "$lib/types";
    import LoadingSkeleton from "$lib/components/LoadingSkeleton.svelte";
    import ActionConfirmModal from "$lib/components/ActionConfirmModal.svelte";

    let diagnostics: SearchDiagnostics | null = $state(null);
    let healthDashboard: HealthDashboard | null = $state(null);
    let recoveryHints: RecoveryHint[] = $state([]);
    let configAudit: ConfigAuditEntry[] = $state([]);
    let schemaValidation: ConfigSchemaValidationReport | null = $state(null);
    let usageAnalytics: UsageAnalyticsReport | null = $state(null);
    let supportBundle: SupportBundleReport | null = $state(null);
    let supportBusy = $state(false);
    let resetBusy = $state(false);
    let showFactoryResetConfirm = $state(false);
    let loading = $state(true);
    let lastUpdated = $state<Date | null>(null);

    async function loadDiagnostics() {
        try { diagnostics = await invoke<SearchDiagnostics>("get_search_diagnostics"); }
        catch (e) { console.error("Failed to load diagnostics:", e); }
    }

    async function loadHealthDashboard() {
        try { healthDashboard = await invoke<HealthDashboard>("get_health_dashboard"); }
        catch (e) { console.error("Failed to load health dashboard:", e); }
    }

    async function loadRecoveryHints() {
        try { recoveryHints = await invoke<RecoveryHint[]>("get_recovery_hints"); }
        catch (e) { console.error("Failed to load recovery hints:", e); }
    }

    async function loadConfigAudit() {
        try { configAudit = await invoke<ConfigAuditEntry[]>("get_config_audit", { limit: 40 }); }
        catch (e) { console.error("Failed to load config audit:", e); }
    }

    async function loadSchemaValidation() {
        try { schemaValidation = await invoke<ConfigSchemaValidationReport>("validate_config_schema"); }
        catch (e) { console.error("Failed to validate config schema:", e); }
    }

    async function loadUsageAnalytics() {
        try { usageAnalytics = await invoke<UsageAnalyticsReport>("get_usage_analytics"); }
        catch (e) { console.error("Failed to load usage analytics:", e); }
    }

    async function buildSupportBundle() {
        supportBusy = true;
        try {
            supportBundle = await invoke<SupportBundleReport>("create_support_bundle", { outputPath: null });
            await Promise.all([loadHealthDashboard(), loadRecoveryHints(), loadConfigAudit(), loadSchemaValidation(), loadUsageAnalytics()]);
        } catch (e) { console.error("Failed to create support bundle:", e); }
        finally { supportBusy = false; }
    }

    async function runFactoryReset() {
        resetBusy = true;
        try {
            await invoke("factory_reset_config");
            supportBundle = null;
            await Promise.all([loadDiagnostics(), loadHealthDashboard(), loadRecoveryHints(), loadConfigAudit(), loadSchemaValidation(), loadUsageAnalytics()]);
        } catch (e) {
            console.error("Failed to factory reset config:", e);
        } finally {
            resetBusy = false;
            showFactoryResetConfirm = false;
        }
    }

    async function refreshAll() {
        loading = true;
        await Promise.all([loadDiagnostics(), loadHealthDashboard(), loadRecoveryHints(), loadConfigAudit(), loadSchemaValidation(), loadUsageAnalytics()]);
        lastUpdated = new Date();
        loading = false;
    }

    onMount(async () => {
        await refreshAll();
    });

    function statusTone(status: string): "ok" | "warn" | "bad" {
        const value = status.toLowerCase();
        if (value.includes("ok") || value.includes("healthy") || value.includes("pass")) return "ok";
        if (value.includes("warn") || value.includes("degrad") || value.includes("slow")) return "warn";
        return "bad";
    }

    function buildSparkline(values: number[], width = 220, height = 48): string {
        if (!values.length) return "";
        if (values.length === 1) return `M0 ${height / 2} L${width} ${height / 2}`;
        const min = Math.min(...values);
        const max = Math.max(...values);
        const span = Math.max(1, max - min);
        const step = width / (values.length - 1);
        return values
            .map((v, i) => {
                const x = i * step;
                const y = height - ((v - min) / span) * height;
                return `${i === 0 ? "M" : "L"}${x.toFixed(2)} ${y.toFixed(2)}`;
            })
            .join(" ");
    }

    let healthRisk = $derived.by(() => {
        if (!healthDashboard) return { ok: 0, warn: 0, bad: 0 };
        const summary = { ok: 0, warn: 0, bad: 0 };
        for (const check of healthDashboard.checks) summary[statusTone(check.status)] += 1;
        return summary;
    });

    let usageHourSparkline = $derived.by(() => {
        if (!usageAnalytics?.peak_usage_hours?.length) return "";
        const sorted = [...usageAnalytics.peak_usage_hours].sort((a, b) => a.hour - b.hour);
        return buildSparkline(sorted.map((h) => h.launches));
    });

    let appFrecencySparkline = $derived.by(() => {
        if (!usageAnalytics?.most_used_apps?.length) return "";
        return buildSparkline(usageAnalytics.most_used_apps.slice(0, 8).map((a) => a.frecency));
    });

</script>

{#if loading}
    <LoadingSkeleton lines={4} />
{:else}
    <div class="control-group control-group-block v2-card">
        <h4>Actions</h4>
        <div class="preset-row">
            <button class="preset-btn btn-secondary" onclick={refreshAll}>Refresh All</button>
            <button class="preset-btn" onclick={loadDiagnostics}>Refresh Metrics</button>
            <button class="preset-btn" onclick={loadHealthDashboard}>Refresh Health</button>
            <button class="preset-btn" onclick={loadRecoveryHints}>Refresh Hints</button>
            <button class="preset-btn" onclick={loadConfigAudit}>Refresh Audit</button>
            <button class="preset-btn" onclick={loadSchemaValidation}>Validate Schema</button>
            <button class="preset-btn" onclick={loadUsageAnalytics}>Refresh Analytics</button>
            <button class="preset-btn" onclick={buildSupportBundle} disabled={supportBusy}>
                {supportBusy ? "Building..." : "Create Support Bundle"}
            </button>
            <button class="preset-btn danger-outline" onclick={() => (showFactoryResetConfirm = true)} disabled={resetBusy}>
                {resetBusy ? "Resetting..." : "Factory Reset"}
            </button>
        </div>
        {#if lastUpdated}
            <div class="status-info">Last refreshed: {lastUpdated.toLocaleString()}</div>
        {/if}
    </div>



    {#if diagnostics}
        <div class="control-group control-group-block">
            <h4>Performance</h4>
            <div class="diagnostics-cards">
            <article class="diag-card">
                <span class="diag-label">Search Latency</span>
                <strong>{diagnostics.search.avg_ms.toFixed(1)}ms</strong>
                <span class="diag-sub">max {diagnostics.search.max_ms.toFixed(1)}ms · {diagnostics.search.calls} calls</span>
            </article>
            <article class="diag-card">
                <span class="diag-label">Suggestion Latency</span>
                <strong>{diagnostics.suggestions.avg_ms.toFixed(1)}ms</strong>
                <span class="diag-sub">max {diagnostics.suggestions.max_ms.toFixed(1)}ms · {diagnostics.suggestions.calls} calls</span>
            </article>
            <article class="diag-card">
                <span class="diag-label">Launch Latency</span>
                <strong>{diagnostics.launch.avg_ms.toFixed(1)}ms</strong>
                <span class="diag-sub">max {diagnostics.launch.max_ms.toFixed(1)}ms · {diagnostics.launch.calls} calls</span>
            </article>
            </div>
        </div>
    {/if}

    {#if healthDashboard}
        <div class="control-group control-group-block">
            <h4>System Snapshot</h4>
            <label>Active Profile <input type="text" value={healthDashboard.active_profile_id} readonly /></label>
            <label>Config Schema <input type="text" value={`${healthDashboard.config_schema}`} readonly /></label>
            <label>Indexed Entries <input type="text" value={`${healthDashboard.file_index_entries}`} readonly /></label>
            <label>Apps Cached <input type="text" value={`${healthDashboard.apps_cached}`} readonly /></label>
            <label>Extensions Cached <input type="text" value={`${healthDashboard.extensions_cached}`} readonly /></label>
            <label>Macro Jobs <input type="text" value={`${healthDashboard.macro_jobs_total}`} readonly /></label>
        </div>

        <div class="control-group control-group-block">
            <h4>Health Risk</h4>
            <div class="diagnostics-cards">
                <article class="diag-card">
                    <span class="diag-label">Healthy<span class="diag-check-badge tone-ok">OK</span></span>
                    <strong>{healthRisk.ok}</strong>
                    <span class="diag-sub">Stable subsystems</span>
                </article>
                <article class="diag-card warn">
                    <span class="diag-label">Warnings<span class="diag-check-badge tone-warn">WARN</span></span>
                    <strong>{healthRisk.warn}</strong>
                    <span class="diag-sub">Needs monitoring</span>
                </article>
                <article class="diag-card danger">
                    <span class="diag-label">High Risk<span class="diag-check-badge tone-bad">RISK</span></span>
                    <strong>{healthRisk.bad}</strong>
                    <span class="diag-sub">Investigate immediately</span>
                </article>
            </div>
        </div>

        <div class="control-group control-group-block">
            <h4>Health Checks</h4>
            <ul class="hint-list">
                {#each healthDashboard.checks as check}
                    <li>
                        <strong>{check.name}</strong>
                        <span class={`diag-check-badge tone-${statusTone(check.status)}`}>{check.status}</span>
                        - {check.detail}
                    </li>
                {/each}
            </ul>
        </div>
    {/if}

    <div class="control-group control-group-block">
        <h4>Recovery Hints</h4>
        <ul class="hint-list">
            {#each recoveryHints as hint}
                <li><strong>{hint.title}</strong> - {hint.detail}</li>
            {/each}
        </ul>
    </div>

    <div class="control-group control-group-block">
        <h4>Config Audit (Recent)</h4>
        <ul class="hint-list">
            {#if configAudit.length === 0}
                <li>No config mutations recorded yet.</li>
            {:else}
                {#each [...configAudit].reverse() as entry}
                    <li>
                        <strong>{new Date(entry.timestamp_ms).toLocaleString()}</strong>
                        [{entry.source}] - {entry.diff.length} change{entry.diff.length === 1 ? "" : "s"}
                        {#if entry.diff.length > 0}
                            <div class="audit-diff-paths">
                                {#each entry.diff.slice(0, 8) as item}
                                    <code>{item.path || "(root)"}</code>
                                {/each}
                                {#if entry.diff.length > 8}
                                    <span>+{entry.diff.length - 8} more</span>
                                {/if}
                            </div>
                        {/if}
                    </li>
                {/each}
            {/if}
        </ul>
    </div>

    {#if schemaValidation}
        <div class="control-group control-group-block">
            <h4>Schema Validation</h4>
            <p>
                Status:
                <strong class:schema-ok={schemaValidation.valid} class:schema-bad={!schemaValidation.valid}>
                    {schemaValidation.valid ? "Valid" : "Invalid"}
                </strong>
            </p>
            {#if !schemaValidation.valid && schemaValidation.errors.length > 0}
                <ul class="hint-list">
                    {#each schemaValidation.errors.slice(0, 12) as err}
                        <li>{err}</li>
                    {/each}
                    {#if schemaValidation.errors.length > 12}
                        <li>+{schemaValidation.errors.length - 12} more error(s)</li>
                    {/if}
                </ul>
            {/if}
        </div>
    {/if}

    {#if usageAnalytics}
        <div class="control-group control-group-block">
            <h4>Usage Analytics (Local Only)</h4>
            <p>Total launch events: <strong>{usageAnalytics.total_launch_events}</strong></p>
            <p>Generated: {new Date(usageAnalytics.generated_at).toLocaleString()}</p>
        </div>


        <div class="control-group control-group-block">
            <h4>Trends</h4>
            <div class="diagnostics-cards">
                <article class="diag-card wide">
                    <span class="diag-label">Hourly Usage</span>
                    {#if usageHourSparkline}
                        <svg viewBox="0 0 220 48" class="diag-sparkline" aria-label="Hourly usage sparkline">
                            <path d={usageHourSparkline} fill="none" stroke="var(--ds-accent, #7b35f0)" stroke-width="2" stroke-linecap="round" />
                        </svg>
                    {:else}
                        <span class="diag-sub">No hourly data yet.</span>
                    {/if}
                </article>
                <article class="diag-card wide">
                    <span class="diag-label">App Frecency</span>
                    {#if appFrecencySparkline}
                        <svg viewBox="0 0 220 48" class="diag-sparkline" aria-label="App frecency sparkline">
                            <path d={appFrecencySparkline} fill="none" stroke="var(--ds-info, #58a6ff)" stroke-width="2" stroke-linecap="round" />
                        </svg>
                    {:else}
                        <span class="diag-sub">No app usage trend yet.</span>
                    {/if}
                </article>
            </div>
        </div>

        <div class="control-group control-group-block">
            <h4>Most Used Apps</h4>
            <ul class="hint-list">
                {#if usageAnalytics.most_used_apps.length === 0}
                    <li>No app launch history yet.</li>
                {:else}
                    {#each usageAnalytics.most_used_apps as item}
                        <li><strong>{item.exec}</strong> - launches: {item.launches}, frecency: {item.frecency}</li>
                    {/each}
                {/if}
            </ul>
        </div>

        <div class="control-group control-group-block">
            <h4>Peak Usage Hours</h4>
            <ul class="hint-list">
                {#if usageAnalytics.peak_usage_hours.length === 0}
                    <li>No hourly launch data yet.</li>
                {:else}
                    {#each usageAnalytics.peak_usage_hours as hour}
                        <li><strong>{hour.hour.toString().padStart(2, "0")}:00</strong> - {hour.launches} launch event{hour.launches === 1 ? "" : "s"}</li>
                    {/each}
                {/if}
            </ul>
        </div>

        <div class="control-group control-group-block">
            <h4>Search Patterns</h4>
            <ul class="hint-list">
                {#if usageAnalytics.search_patterns.length === 0}
                    <li>No query pattern data yet.</li>
                {:else}
                    {#each usageAnalytics.search_patterns as pattern}
                        <li><strong>{pattern.query}</strong> - {pattern.count}</li>
                    {/each}
                {/if}
            </ul>
        </div>
    {/if}

    {#if supportBundle}
        <div class="control-group control-group-block">
            <label>Latest Support Bundle <input type="text" value={supportBundle.path} readonly /></label>
            <label>Size (bytes) <input type="text" value={`${supportBundle.size_bytes}`} readonly /></label>
        </div>
    {/if}

    {#if showFactoryResetConfirm}
        <ActionConfirmModal
            title="Factory reset settings?"
            description="This resets all Vanta settings to defaults immediately. This action cannot be undone."
            confirmLabel="Factory Reset"
            cancelLabel="Cancel"
            onConfirm={runFactoryReset}
            onCancel={() => (showFactoryResetConfirm = false)}
            busy={resetBusy}
        />
    {/if}
{/if}

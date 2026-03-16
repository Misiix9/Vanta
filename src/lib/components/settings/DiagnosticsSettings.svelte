<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import type { SearchDiagnostics, HealthDashboard, RecoveryHint, SupportBundleReport } from "$lib/types";
    import LoadingSkeleton from "$lib/components/LoadingSkeleton.svelte";
    import ActionConfirmModal from "$lib/components/ActionConfirmModal.svelte";

    let diagnostics: SearchDiagnostics | null = $state(null);
    let healthDashboard: HealthDashboard | null = $state(null);
    let recoveryHints: RecoveryHint[] = $state([]);
    let supportBundle: SupportBundleReport | null = $state(null);
    let supportBusy = $state(false);
    let resetBusy = $state(false);
    let showFactoryResetConfirm = $state(false);
    let loading = $state(true);

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

    async function buildSupportBundle() {
        supportBusy = true;
        try {
            supportBundle = await invoke<SupportBundleReport>("create_support_bundle", { outputPath: null });
            await loadHealthDashboard();
            await loadRecoveryHints();
        } catch (e) { console.error("Failed to create support bundle:", e); }
        finally { supportBusy = false; }
    }

    async function runFactoryReset() {
        resetBusy = true;
        try {
            await invoke("factory_reset_config");
            supportBundle = null;
            await Promise.all([loadDiagnostics(), loadHealthDashboard(), loadRecoveryHints()]);
        } catch (e) {
            console.error("Failed to factory reset config:", e);
        } finally {
            resetBusy = false;
            showFactoryResetConfirm = false;
        }
    }

    onMount(async () => {
        await Promise.all([loadDiagnostics(), loadHealthDashboard(), loadRecoveryHints()]);
        loading = false;
    });
</script>

{#if loading}
    <LoadingSkeleton lines={4} />
{:else}
    <div class="control-group">
        <button class="close-btn" onclick={loadDiagnostics}>Refresh Metrics</button>
        <button class="close-btn" onclick={loadHealthDashboard}>Refresh Health</button>
        <button class="close-btn" onclick={loadRecoveryHints}>Refresh Hints</button>
        <button class="close-btn" onclick={buildSupportBundle} disabled={supportBusy}>
            {supportBusy ? "Building..." : "Create Support Bundle"}
        </button>
        <button class="close-btn danger-outline" onclick={() => (showFactoryResetConfirm = true)} disabled={resetBusy}>
            {resetBusy ? "Resetting..." : "Factory Reset"}
        </button>
    </div>

    {#if diagnostics}
        <div class="control-group">
            <label>Search Calls <input type="text" value={`${diagnostics.search.calls}`} readonly /></label>
            <label>Search Avg (ms) <input type="text" value={diagnostics.search.avg_ms.toFixed(2)} readonly /></label>
            <label>Search Max (ms) <input type="text" value={`${diagnostics.search.max_ms}`} readonly /></label>
        </div>
        <div class="control-group">
            <label>Suggestion Calls <input type="text" value={`${diagnostics.suggestions.calls}`} readonly /></label>
            <label>Suggestion Avg (ms) <input type="text" value={diagnostics.suggestions.avg_ms.toFixed(2)} readonly /></label>
            <label>Launch Avg (ms) <input type="text" value={diagnostics.launch.avg_ms.toFixed(2)} readonly /></label>
        </div>
    {/if}

    {#if healthDashboard}
        <div class="control-group">
            <label>Active Profile <input type="text" value={healthDashboard.active_profile_id} readonly /></label>
            <label>Config Schema <input type="text" value={`${healthDashboard.config_schema}`} readonly /></label>
            <label>Indexed Entries <input type="text" value={`${healthDashboard.file_index_entries}`} readonly /></label>
        </div>
        <div class="control-group">
            <label>Apps Cached <input type="text" value={`${healthDashboard.apps_cached}`} readonly /></label>
            <label>Extensions Cached <input type="text" value={`${healthDashboard.extensions_cached}`} readonly /></label>
            <label>Macro Jobs <input type="text" value={`${healthDashboard.macro_jobs_total}`} readonly /></label>
        </div>
        <div class="control-group control-group-block">
            <h4>Health Checks</h4>
            <ul class="hint-list">
                {#each healthDashboard.checks as check}
                    <li><strong>{check.name}</strong> [{check.status}] - {check.detail}</li>
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

<style>
    .danger-outline {
        border-color: color-mix(in srgb, var(--ds-danger, #d44) 45%, transparent);
        color: var(--ds-danger, #d44);
    }
</style>

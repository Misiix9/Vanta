<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import type {
    VantaConfig,
    SearchResult,
    CommandContract,
    ThemeMeta,
    WorkflowMacro,
    MacroDryRunResult,
    ExtensionEntry,
    Capability,
    PermissionNeededPayload,
    ResultAction,
  } from "$lib/types";
  import {
    composeResults,
    commandForResult,
    commandToExec,
    inferCommandFromResult,
    execForAction,
    getConfirmSpec,
    normalizeSettingsSection,
    parsePermissionError,
  } from "$lib/ranking";
  import SearchInput from "$lib/components/SearchInput.svelte";
  import ResultsList from "$lib/components/ResultsList.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  import NotificationCenter from "$lib/components/NotificationCenter.svelte";
  import MacroPreview from "$lib/components/MacroPreview.svelte";
  import NowPlayingBar from "$lib/components/NowPlayingBar.svelte";
  import FirstRunWizard from "$lib/components/FirstRunWizard.svelte";
  import QuickTipsPanel from "$lib/components/QuickTipsPanel.svelte";
  import SearchExplainPanel from "$lib/components/SearchExplainPanel.svelte";
  import ActionConfirmModal from "$lib/components/ActionConfirmModal.svelte";
  import KeyboardShortcutsModal from "$lib/components/KeyboardShortcutsModal.svelte";
  import PreviewPanel from "$lib/components/PreviewPanel.svelte";
  import ContextMenu from "$lib/components/ContextMenu.svelte";
  import type { ToastOptions } from "$lib/sdk/types";
  import { toastHistory } from "$lib/stores/toastStore";

  type ViewId = "launcher" | "settings" | "store" | "featureHub" | "communityHub" | "themeHub" | "extensionsHub";

  let {
    config = $bindable<VantaConfig | undefined>(),
    availableExtensions = [],
    availableMacros = [],
    availableThemes = [],
    blurMode = "fallback",
    permissionPrompt = $bindable<{
      scriptId: string;
      missingCaps: Capability[];
      requestedCaps: Capability[];
    } | null>(),
    permissionBusy = false,
    clipboardHistory = $bindable<any[]>(),
    currentMode = $bindable<"launcher" | "clipboard">(),
    view = $bindable<ViewId>(),
    settingsStartSection = $bindable<string | null>(),
    extensionView = $bindable<{ extId: string; command: string; extPath: string } | null>(),
    onToast,
    onResetAndHide,
    onSaveConfigUpdate,
  }: {
    config: VantaConfig | undefined;
    availableExtensions: ExtensionEntry[];
    availableMacros: WorkflowMacro[];
    availableThemes: ThemeMeta[];
    blurMode: string;
    permissionPrompt: { scriptId: string; missingCaps: Capability[]; requestedCaps: Capability[] } | null;
    permissionBusy: boolean;
    clipboardHistory: any[];
    currentMode: "launcher" | "clipboard";
    view: ViewId;
    settingsStartSection: string | null;
    extensionView: { extId: string; command: string; extPath: string } | null;
    onToast: (options: ToastOptions) => void;
    onResetAndHide: () => void;
    onSaveConfigUpdate: (mutator: (cfg: VantaConfig) => void) => Promise<void>;
  } = $props();

  let query = $state("");
  let results: SearchResult[] = $state([]);
  let baseResults: SearchResult[] = $state([]);
  let selectedIndex = $state(0);
  let searchTime: number | null = $state(null);
  let visibleRowCount = $state(0);
  let searchRequestId = 0;
  let actionInFlight = $state(false);
  let searchInputRef: SearchInput | undefined = $state();
  let resultsListRef: ResultsList | undefined = $state();
  let pendingScrollFrame: number | null = null;
  let isSearching = $state(false);
  let queryHistory: string[] = $state([]);

  let activeMacroId = $state<string | null>(null);
  let macroArgs = $state<Record<string, string>>({});
  let macroDryRun = $state<MacroDryRunResult | null>(null);
  let macroBusy = $state(false);
  let macroError = $state<string | null>(null);
  let currentMacro = $derived(
    activeMacroId ? availableMacros.find((m) => m.id === activeMacroId) : null,
  );

  const ONBOARDING_SEEN_KEY = "vanta.onboarding.v2_3_seen";
  const QUICK_TIPS_HIDDEN_KEY = "vanta.quick_tips.hidden";
  let onboardingOpen = $state(false);
  let quickTipsVisible = $state(false);
  let pendingConfirmResult = $state<SearchResult | null>(null);
  let shortcutsOpen = $state(false);
  let previewResult = $state<SearchResult | null>(null);
  let contextMenu = $state<{ result: SearchResult; x: number; y: number } | null>(null);
  let showScoreOverlay = $state(false);
  let showNotifications = $state(false);
  let groupBySection = $derived(query.trim() === "");
  let totalItems = $derived(visibleRowCount);
  let notificationCount = $derived($toastHistory.length);
  let workspaceLayout: "single" | "multi" = $derived(config?.search.layout_mode === "multi" ? "multi" : "single");
  let selectedResult = $derived.by(() => {
    const row = resultsListRef?.getVisibleRow(selectedIndex);
    return row?.type === "item" ? row.result : null;
  });

  $effect(() => {
    if (extensionView || currentMode !== "launcher") return;
    const extVersion = availableExtensions.length;
    results = composeResults(baseResults, query, availableMacros, config);
    void extVersion;
  });

  $effect(() => {
    const count = visibleRowCount;
    if (count === 0) { selectedIndex = 0; return; }
    if (selectedIndex >= count) selectedIndex = count - 1;
  });

  let partialUnlisten: (() => void) | null = null;

  onMount(async () => {
    onboardingOpen = localStorage.getItem(ONBOARDING_SEEN_KEY) !== "1";
    quickTipsVisible = localStorage.getItem(QUICK_TIPS_HIDDEN_KEY) !== "1";

    try { queryHistory = await invoke<string[]>("get_query_history"); } catch (_) {}

    partialUnlisten = await listen<SearchResult[]>("search-partial", (event) => {
      if (query.trim() && event.payload.length > 0) {
        // Merge partial results into current display for progressive rendering.
        const partial = event.payload;
        const existingIds = new Set(baseResults.map((r) => r.exec));
        const fresh = partial.filter((r) => !existingIds.has(r.exec));
        if (fresh.length > 0) {
          baseResults = [...baseResults, ...fresh].sort((a, b) => b.score - a.score);
          results = composeResults(baseResults, query, availableMacros, config);
        }
      }
    });

    if (currentMode === "launcher" && query.trim() === "") {
      await loadSuggestions();
    }
  });

  onDestroy(() => {
    partialUnlisten?.();
  });

  function scheduleScrollToSelected() {
    if (pendingScrollFrame !== null) return;
    pendingScrollFrame = requestAnimationFrame(() => {
      pendingScrollFrame = null;
      resultsListRef?.scrollToSelected();
    });
  }

  export function focus() { searchInputRef?.focus?.(); }

  export function refreshSuggestions() {
    if (query.trim() === "" && currentMode === "launcher") loadSuggestions();
    else if (currentMode === "launcher") handleSearch(query);
  }

  function handlePermissionError(err: unknown): boolean {
    const parsed = parsePermissionError(err);
    if (!parsed) return false;
    if (parsed.type === "needed") {
      permissionPrompt = {
        scriptId: parsed.payload.script_id,
        missingCaps: parsed.payload.missing_caps || [],
        requestedCaps: parsed.payload.requested_caps || parsed.payload.missing_caps || [],
      };
      return true;
    }
    return false;
  }

  function resetMacroState() {
    activeMacroId = null; macroArgs = {}; macroDryRun = null; macroBusy = false; macroError = null;
  }

  async function dryRunMacro(macro: WorkflowMacro) {
    macroBusy = true; macroError = null;
    try { macroDryRun = await invoke<MacroDryRunResult>("dry_run_macro", { macroId: macro.id, args: macroArgs }); }
    catch (e) {
      macroError = String(e); console.error("Dry-run failed", e);
      onToast({ title: "Macro Dry Run Failed", message: String(e), type: "error" });
    }
    finally { macroBusy = false; }
  }

  async function runMacro(macro: WorkflowMacro) {
    macroBusy = true; macroError = null;
    try {
      await invoke("start_macro_job", { macroId: macro.id, args: macroArgs });
      resetMacroState(); onResetAndHide();
    } catch (e) {
      if (handlePermissionError(e)) { macroBusy = false; return; }
      macroError = String(e); console.error("Run macro failed", e);
      onToast({ title: "Macro Run Failed", message: String(e), type: "error" });
    } finally { macroBusy = false; }
  }

  async function loadSuggestions() {
    const requestId = ++searchRequestId;
    try {
      const suggestions = await invoke<SearchResult[]>("get_suggestions_v3");
      if (requestId !== searchRequestId) return;
      if (suggestions.length === 0) {
        const fallback = await invoke<SearchResult[]>("search_v3", { query: "" });
        if (requestId !== searchRequestId) return;
        baseResults = fallback;
      } else { baseResults = suggestions; }
      results = composeResults(baseResults, "", availableMacros, config);
      selectedIndex = results.length > 0 ? 1 : 0;
    } catch (e) {
      console.error("Failed to load suggestions:", e);
      onToast({ title: "Suggestions Failed", message: String(e), type: "error" });
      try {
        const fallback = await invoke<SearchResult[]>("search_v3", { query: "" });
        if (requestId === searchRequestId) {
          baseResults = fallback;
          results = composeResults(baseResults, "", availableMacros, config);
          if (results.length > 0) selectedIndex = 1;
        }
      } catch (err) {
        console.error("Fallback search failed:", err);
        onToast({ title: "Search Failed", message: String(err), type: "error" });
      }
    }
  }

  async function handleSearch(q: string) {
    const requestId = ++searchRequestId;
    if (!q.trim()) { resetMacroState(); searchTime = null; isSearching = false; await loadSuggestions(); return; }
    resetMacroState();
    isSearching = true;
    try {
      const start = performance.now();
      const searchResults = await invoke<SearchResult[]>("search_v3", { query: q });
      if (requestId !== searchRequestId) return;
      baseResults = searchResults;
      results = composeResults(baseResults, q, availableMacros, config);
      selectedIndex = 0;
      searchTime = performance.now() - start;
      // Save non-trivial queries to history
      if (q.trim().length >= 2) {
        invoke("save_query_history", { query: q.trim() }).then((r) => {
          invoke<string[]>("get_query_history").then((h) => { queryHistory = h; }).catch(() => {});
        }).catch(() => {});
      }
    } catch (e) {
      if (requestId !== searchRequestId) return;
      baseResults = []; results = []; searchTime = null;
    } finally {
      if (requestId === searchRequestId) isSearching = false;
    }
  }

  async function executeIntentWorkflow(steps: string[]) {
    for (const stepExec of steps) {
      const stepResult: SearchResult = {
        title: "Intent Step", subtitle: null, icon: null, exec: stepExec,
        score: 0, match_indices: [], source: "Application",
      };
      const cmd = inferCommandFromResult(stepResult);
      if (cmd.kind === "open_settings") { settingsStartSection = null; view = "settings"; continue; }
      if (cmd.kind === "open_settings_section") { settingsStartSection = normalizeSettingsSection(cmd.section); view = "settings"; continue; }
      if (cmd.kind === "open_feature_window") {
        const t = cmd.window;
        if (t === "featureHub" || t === "communityHub" || t === "themeHub" || t === "extensionsHub") view = t;
        continue;
      }
      if (cmd.kind === "open_store") { view = "store"; continue; }
      if (cmd.kind === "query_fill") { query = cmd.value; await handleSearch(query); continue; }
      if (cmd.kind === "system_action") { await invoke("system_action", { action: cmd.action }); continue; }
      if (cmd.kind === "launch_app") { await invoke("launch_app", { exec: cmd.exec }); }
    }
  }

  async function handleActivate(result: SearchResult, bypassConfirm = false) {
    if (permissionPrompt) return;
    if (!bypassConfirm) {
      const confirm = getConfirmSpec(result);
      if (confirm) { pendingConfirmResult = result; return; }
    }
    if (actionInFlight) return;
    actionInFlight = true;
    try {
      const command = commandForResult(result);
      if (command.kind === "macro_open") {
        const macro = availableMacros.find((m) => m.id === command.id);
        if (macro) {
          activeMacroId = command.id;
          macroArgs = Object.fromEntries((macro.args ?? []).map((a) => [a.name, a.default_value ?? ""]));
          macroDryRun = null; macroError = null;
        }
        return;
      }
      if (command.kind === "extension_view" || command.kind === "extension_action") {
        const ext = availableExtensions.find((e) => e.manifest.name === command.ext_id);
        extensionView = { extId: command.ext_id, command: command.command, extPath: ext?.path ?? "" };
        return;
      }
      if (command.kind === "query_fill") {
        query = command.value; await handleSearch(query);
        requestAnimationFrame(() => requestAnimationFrame(() => searchInputRef?.focus?.()));
        return;
      }
      if (command.kind === "intent_workflow") { await executeIntentWorkflow(command.steps); return; }
      if (command.kind === "profile_switch") {
        try {
          const updated = await invoke<VantaConfig>("switch_profile", { profileId: command.id });
          await onSaveConfigUpdate((cfg) => Object.assign(cfg, updated));
          await loadSuggestions(); searchInputRef?.focus?.();
          onToast({ title: "Profile Switched", message: command.id, type: "success" });
        } catch (e) {
          console.error("Profile switch failed", e);
          onToast({ title: "Profile Switch Failed", message: String(e), type: "error" });
        }
        return;
      }
      if (command.kind === "system_action") {
        try {
          await invoke("system_action", { action: command.action });
          onToast({ title: "System Action", message: `${command.action} executed`, type: "success" });
          onResetAndHide();
        } catch (e) {
          console.error("System action failed", e);
          onToast({ title: "System Action Failed", message: String(e), type: "error" });
        }
        return;
      }
      if (command.kind === "open_settings") { settingsStartSection = null; view = "settings"; return; }
      if (command.kind === "open_settings_section") { settingsStartSection = normalizeSettingsSection(command.section); view = "settings"; return; }
      if (command.kind === "open_feature_window") {
        const t = command.window;
        if (t === "featureHub" || t === "communityHub" || t === "themeHub" || t === "extensionsHub") { view = t; return; }
        view = "featureHub"; return;
      }
      if (command.kind === "open_store") { view = "store"; return; }
      if (command.kind === "copy_path") {
        await navigator.clipboard.writeText(command.value);
        onToast({ title: "Path Copied", message: command.value, type: "success" });
        onResetAndHide();
      }
      else if (command.kind === "reveal_path") {
        await invoke("reveal_in_file_manager", { path: command.path });
        onToast({ title: "Opened In File Manager", message: command.path, type: "success" });
        onResetAndHide();
      }
      else if (command.kind === "open_with_editor") {
        await invoke("open_with_editor", { path: command.path });
        onToast({ title: "Opened In Editor", message: command.path, type: "success" });
        onResetAndHide();
      }
      else if (command.kind === "copy_text") {
        await navigator.clipboard.writeText(command.value);
        onToast({ title: "Copied", message: command.value, type: "success" });
        onResetAndHide();
      }
      else if (command.kind === "open_file") {
        await invoke("open_path", { path: command.path });
        onToast({ title: "Opening File", message: command.path, type: "success" });
        onResetAndHide();
      }
      else if (command.kind === "launch_app" && command.exec.startsWith("note-save:")) {
        await invoke("launch_app", { exec: command.exec });
        onToast({ title: "Note Saved", message: command.exec.slice(10), type: "success" });
        onResetAndHide();
      }
      else if (command.kind === "launch_app" && command.exec.startsWith("bookmark-add:")) {
        const path = command.exec.slice(13);
        await invoke("launch_app", { exec: command.exec });
        onToast({ title: "Bookmarked", message: path, type: "success" });
      }
      else if (command.kind === "launch_app" && command.exec.startsWith("bookmark-remove:")) {
        const path = command.exec.slice(16);
        await invoke("launch_app", { exec: command.exec });
        onToast({ title: "Bookmark Removed", message: path, type: "success" });
      }
      else {
        await invoke("launch_app", { exec: commandToExec(command) });
        onToast({ title: "Launching", message: result.title, type: "success" });
        onResetAndHide();
      }
    } catch (e) {
      console.error("Launch/Copy failed:", e);
      onToast({ title: "Action Failed", message: String(e), type: "error" });
    }
    finally { actionInFlight = false; }
  }

  function handleEscape() {
    if (permissionPrompt) return;
    if (contextMenu) { contextMenu = null; return; }
    if (previewResult) { previewResult = null; return; }
    if (extensionView) { extensionView = null; return; }
    if (activeMacroId) { resetMacroState(); return; }
    onResetAndHide();
  }

  function handleActionClick(result: SearchResult, action: ResultAction) {
    const exec = execForAction(result, action);
    handleActivate({ ...result, exec, command: action.command }, false);
  }

  function handleContextMenuAction(result: SearchResult, exec: string, command?: ResultAction["command"]) {
    handleActivate({ ...result, exec, command }, false);
  }

  function handleResultContextMenu(e: MouseEvent, result: SearchResult) {
    contextMenu = { result, x: e.clientX, y: e.clientY };
  }

  function handleHistoryRecall(q: string) {
    query = q;
    handleSearch(q);
  }

  function completeOnboarding() {
    onboardingOpen = false;
    localStorage.setItem(ONBOARDING_SEEN_KEY, "1");
  }

  function dismissQuickTips() {
    quickTipsVisible = false;
    localStorage.setItem(QUICK_TIPS_HIDDEN_KEY, "1");
  }

  function applyTipQuery(text: string) {
    query = text; void handleSearch(text);
    requestAnimationFrame(() => searchInputRef?.focus?.());
  }

  async function toggleWorkspaceLayout() {
    const next = workspaceLayout === "multi" ? "single" : "multi";
    if (!config) return;
    await onSaveConfigUpdate((cfg) => {
      cfg.search.layout_mode = next;
      const profiles = cfg.profiles;
      if (profiles?.entries?.length) {
        const active = profiles.entries.find((p) => p.id === profiles.active_profile_id);
        if (active) active.search.layout_mode = next;
      }
    });
  }

  function sourceLabel(source: SearchResult["source"]): string {
    if (typeof source === "object") return `Extension (${source.Extension.ext_id})`;
    return source;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (shortcutsOpen) return;
    if (pendingConfirmResult) {
      if (e.key === "Escape") { e.preventDefault(); pendingConfirmResult = null; }
      return;
    }
    if (onboardingOpen || permissionPrompt) return;
    if (e.key === "?" && (e.ctrlKey || e.metaKey)) { e.preventDefault(); shortcutsOpen = true; return; }
    if (e.key === "d" && (e.ctrlKey || e.metaKey)) { e.preventDefault(); showScoreOverlay = !showScoreOverlay; return; }
    if (activeMacroId && e.key === "Escape") { resetMacroState(); return; }
    if (e.key === "," && e.ctrlKey) { e.preventDefault(); view = view === "launcher" ? "settings" : "launcher"; return; }
    if (e.ctrlKey && e.key === "\\") { e.preventDefault(); void toggleWorkspaceLayout(); return; }
    if (view === "settings" || view === "store") {
      if (e.key === "Escape") { e.preventDefault(); e.stopPropagation(); view = "launcher"; }
      return;
    }
    if (!e.ctrlKey && !e.metaKey && !e.altKey && e.key.length === 1 && document.activeElement?.tagName !== "INPUT") {
      searchInputRef?.focus();
    }
    if (e.key === "Backspace" && document.activeElement?.tagName !== "INPUT") searchInputRef?.focus();
    if (totalItems === 0) return;

    const activeRow = resultsListRef?.getVisibleRow(selectedIndex);
    const activeResult = activeRow?.type === "item" ? activeRow.result : undefined;

    if (activeResult?.actions?.length) {
      const findAction = (prefix: string) => activeResult.actions?.find((a) => execForAction(activeResult, a).startsWith(prefix));
      if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "c") {
        const action = findAction("copy-path:");
        if (action) { e.preventDefault(); handleActivate({ ...activeResult, exec: execForAction(activeResult, action), command: action.command }); return; }
      }
      if (e.key === "Enter" && e.shiftKey) {
        const action = findAction("reveal:") || activeResult.actions![0];
        if (action) { e.preventDefault(); handleActivate({ ...activeResult, exec: execForAction(activeResult, action), command: action.command }); return; }
      }
      if (e.key === "Enter" && e.altKey) {
        const action = findAction("open-with:") || activeResult.actions![0];
        if (action) { e.preventDefault(); handleActivate({ ...activeResult, exec: execForAction(activeResult, action), command: action.command }); return; }
      }
      if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "m") {
        const action = findAction("move-window-current:");
        if (action) { e.preventDefault(); handleActivate({ ...activeResult, exec: execForAction(activeResult, action), command: action.command }); return; }
      }
      if (e.ctrlKey && !e.shiftKey && e.key.toLowerCase() === "m") {
        const action = findAction("minimize-window:");
        if (action) { e.preventDefault(); handleActivate({ ...activeResult, exec: execForAction(activeResult, action), command: action.command }); return; }
      }
    }

    function skipHeaders(idx: number, direction: 1 | -1): number {
      let i = idx; let safety = totalItems;
      while (safety-- > 0 && resultsListRef?.getVisibleRow(i)?.type === "header") {
        i = direction === 1 ? (i + 1) % totalItems : (i <= 0 ? totalItems - 1 : i - 1);
      }
      return i;
    }

    switch (e.key) {
      case "ArrowDown": e.preventDefault(); selectedIndex = skipHeaders((selectedIndex + 1) % totalItems, 1); scheduleScrollToSelected(); break;
      case "ArrowUp": e.preventDefault(); selectedIndex = skipHeaders(selectedIndex <= 0 ? totalItems - 1 : selectedIndex - 1, -1); scheduleScrollToSelected(); break;
      case "Enter": e.preventDefault(); if (activeResult) handleActivate(activeResult); break;
      case " ":
        if (document.activeElement?.tagName !== "INPUT") {
          e.preventDefault();
          if (activeResult) previewResult = previewResult?.exec === activeResult.exec ? null : activeResult;
        }
        break;
      case "Tab":
        e.preventDefault();
        if (results[selectedIndex]?.exec.startsWith("fill:")) { handleActivate(results[selectedIndex]); break; }
        if (e.shiftKey) selectedIndex = skipHeaders(selectedIndex <= 0 ? totalItems - 1 : selectedIndex - 1, -1);
        else selectedIndex = skipHeaders((selectedIndex + 1) % totalItems, 1);
        scheduleScrollToSelected();
        break;
    }
  }
</script>

<svelte:window
  onkeydown={handleKeydown}
  onfocus={() => { if (view !== "settings") searchInputRef?.focus(); }}
/>

<div
  class={`launcher-grid v2-stack density-${config?.accessibility?.spacing_preset ?? "comfortable"}`}
  style="height: 100%; width: 100%;"
>
  <a href="#vanta-results" class="vanta-skip-link">Skip to results</a>
  <SearchInput bind:this={searchInputRef} bind:query onSearch={handleSearch} onEscape={handleEscape} {queryHistory} onHistoryRecall={handleHistoryRecall} />

  {#if !onboardingOpen && quickTipsVisible && query.trim() === ""}
    <QuickTipsPanel onTry={applyTipQuery} onDismiss={dismissQuickTips} />
  {/if}

  {#if currentMacro}
    <MacroPreview
      macro={currentMacro} args={macroArgs} dryRun={macroDryRun}
      busy={macroBusy} error={macroError}
      onArgsChange={(name, value) => (macroArgs = { ...macroArgs, [name]: value })}
      onDryRun={() => dryRunMacro(currentMacro)} onRun={() => runMacro(currentMacro)} onClose={resetMacroState}
    />
  {:else if activeMacroId}
    <div class="empty-state">Macro not found</div>
  {:else if workspaceLayout === "multi"}
    <div class="workspace-canvas">
      <div class="workspace-pane-main">
        <ResultsList
          bind:this={resultsListRef} {results} {groupBySection} bind:selectedIndex
          query={query}
          showScore={showScoreOverlay}
          onActivate={handleActivate}
          onActionClick={handleActionClick}
          onContextMenu={handleResultContextMenu}
          on:visiblecount={(event) => (visibleRowCount = event.detail.count)}
        />
        {#if query.trim() !== "" && config && config.search.show_explain_panel !== false}
          <SearchExplainPanel query={query} results={results} searchConfig={config.search} />
        {/if}
      </div>
      <aside class="workspace-pane-side v2-panel" aria-label="Result details and actions">
        {#if selectedResult}
          <div class="workspace-pane-head">
            <h3>{selectedResult.title}</h3>
            <p>{selectedResult.subtitle ?? "No description"}</p>
            <span class="v2-form-help">{sourceLabel(selectedResult.source)}</span>
          </div>
          <div class="workspace-pane-actions">
            <button class="btn-secondary" onclick={() => handleActivate(selectedResult)}>Open</button>
            <button class="btn-ghost" onclick={() => (previewResult = selectedResult)}>Quick Look</button>
            {#if selectedResult.actions?.length}
              {#each selectedResult.actions as action}
                <button class="btn-ghost" onclick={() => handleActionClick(selectedResult, action)}>{action.label}</button>
              {/each}
            {/if}
          </div>
        {:else}
          <p class="workspace-pane-empty">Select a result to inspect details and actions.</p>
        {/if}
      </aside>
    </div>
  {:else}
    <ResultsList
      bind:this={resultsListRef} {results} {groupBySection} bind:selectedIndex
      query={query}
      showScore={showScoreOverlay}
      onActivate={handleActivate}
      onActionClick={handleActionClick}
      onContextMenu={handleResultContextMenu}
      on:visiblecount={(event) => (visibleRowCount = event.detail.count)}
    />
    {#if query.trim() !== "" && config && config.search.show_explain_panel !== false}
      <SearchExplainPanel query={query} results={results} searchConfig={config.search} />
    {/if}
  {/if}

  <NowPlayingBar onOpenExtension={() => {
    const ext = availableExtensions.find((e) => e.manifest.name === 'spotify');
    if (ext) extensionView = { extId: 'spotify', command: 'player', extPath: ext.path ?? '' };
  }} />
  <StatusBar
    resultCount={activeMacroId ? macroDryRun?.steps.length ?? 0 : visibleRowCount}
    {searchTime}
    {isSearching}
    {notificationCount}
    layoutMode={workspaceLayout}
    onOpenNotifications={() => (showNotifications = true)}
    onToggleLayout={() => void toggleWorkspaceLayout()}
  />

  {#if pendingConfirmResult}
    {@const confirmSpec = getConfirmSpec(pendingConfirmResult)}
    {#if confirmSpec}
      <ActionConfirmModal
        title={confirmSpec.title} description={confirmSpec.description} confirmLabel={confirmSpec.confirmLabel}
        onCancel={() => (pendingConfirmResult = null)}
        onConfirm={() => { const confirmed = pendingConfirmResult; pendingConfirmResult = null; if (confirmed) void handleActivate(confirmed, true); }}
      />
    {/if}
  {/if}

  {#if shortcutsOpen}
    <KeyboardShortcutsModal onClose={() => (shortcutsOpen = false)} />
  {/if}

  {#if previewResult}
    <PreviewPanel result={previewResult} onClose={() => (previewResult = null)} />
  {/if}

  {#if contextMenu}
    <ContextMenu
      result={contextMenu.result}
      x={contextMenu.x}
      y={contextMenu.y}
      onAction={handleContextMenuAction}
      onClose={() => (contextMenu = null)}
    />
  {/if}

  {#if showNotifications}
    <NotificationCenter onClose={() => (showNotifications = false)} />
  {/if}

  {#if onboardingOpen && config}
    <FirstRunWizard
      hotkey={config.general.hotkey} themeId={config.appearance.theme}
      includeHidden={config.files.include_hidden} themes={availableThemes}
      on:tryquery={(event) => applyTipQuery(event.detail.query)}
      on:sethotkey={(event) => void onSaveConfigUpdate((cfg) => { cfg.general.hotkey = event.detail.hotkey; })}
      on:settheme={(event) => void onSaveConfigUpdate((cfg) => { cfg.appearance.theme = event.detail.themeId; })}
      on:setincludehidden={(event) => void onSaveConfigUpdate((cfg) => { cfg.files.include_hidden = event.detail.includeHidden; })}
      on:opensettings={() => { completeOnboarding(); view = "settings"; }}
      on:skip={completeOnboarding} on:complete={completeOnboarding}
    />
  {/if}
</div>

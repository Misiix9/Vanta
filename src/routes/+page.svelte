<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { onDestroy, onMount } from "svelte";
  import { fade } from "svelte/transition";
  import type {
    VantaConfig,
    SearchResult,
    CommandContract,
    BlurStatus,
    ThemeMeta,
    Capability,
    PermissionNeededPayload,
    WorkflowMacro,
    MacroDryRunResult,
    MacroJobRecord,
    ExtensionEntry,
  } from "$lib/types";
  import { applyTheme } from "$lib/theme";
  import SearchInput from "$lib/components/SearchInput.svelte";
  import ResultsList from "$lib/components/ResultsList.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  import SettingsView from "$lib/components/SettingsView.svelte";
  import PermissionModal from "$lib/components/PermissionModal.svelte";
  import MacroPreview from "$lib/components/MacroPreview.svelte";
  import JobsPanel from "$lib/components/JobsPanel.svelte";
  import ExtensionHost from "$lib/components/ExtensionHost.svelte";
  import StoreView from "$lib/components/StoreView.svelte";
  import ClipboardView from "$lib/components/ClipboardView.svelte";
  import NowPlayingBar from "$lib/components/NowPlayingBar.svelte";
  import SpotifyMiniPlayer from "$lib/components/SpotifyMiniPlayer.svelte";
  import FirstRunWizard from "$lib/components/FirstRunWizard.svelte";
  import QuickTipsPanel from "$lib/components/QuickTipsPanel.svelte";
  import SearchExplainPanel from "$lib/components/SearchExplainPanel.svelte";
  import ActionConfirmModal from "$lib/components/ActionConfirmModal.svelte";

  let isMiniPlayer = $state(false);
  let query = $state("");
  let vantaConfig: VantaConfig | undefined = $state();
  let results: SearchResult[] = $state([]);
  let baseResults: SearchResult[] = $state([]);
  let view: "launcher" | "settings" | "store" = $state("launcher");
  let selectedIndex = $state(0);
  let searchTime: number | null = $state(null);
  let visibleRowCount = $state(0);
  let currentMode: "launcher" | "clipboard" = $state("launcher");
  let blurMode: string = $state("fallback");
  let searchInputRef: SearchInput | undefined = $state();
  let resultsListRef: ResultsList | undefined = $state();
  let availableExtensions: ExtensionEntry[] = $state([]);
  let extensionView: { extId: string; command: string; extPath: string } | null = $state(null);
  let availableMacros: WorkflowMacro[] = $state([]);
  let actionInFlight = $state(false); // guard: don't dismiss during actions
  let isVisible = $state(true);
  let isMouseOver = $state(false);
  let permissionPrompt = $state<{
    scriptId: string;
    missingCaps: Capability[];
    requestedCaps: Capability[];
  } | null>(null);
  let permissionBusy = $state(false);
  let activeMacroId = $state<string | null>(null);
  let macroArgs = $state<Record<string, string>>({});
  let macroDryRun = $state<MacroDryRunResult | null>(null);
  let macroBusy = $state(false);
  let macroError = $state<string | null>(null);
  let macroJobs = $state<MacroJobRecord[]>([]);
  let currentMacro = $derived(
    activeMacroId ? availableMacros.find((m) => m.id === activeMacroId) : null,
  );

  // Lightweight app rescan throttle (only when window is shown/opened)
  const RESCAN_INTERVAL_MS = 15000;
  let lastAppRescan = $state(0);

  async function maybeRescanApps(force = false) {
    const now = Date.now();
    if (!force && now - lastAppRescan < RESCAN_INTERVAL_MS) return;
    lastAppRescan = now;
    try {
      await invoke<number>("rescan_apps");
    } catch (e) {
      console.error("App rescan failed", e);
    }
  }

  let availableThemes: ThemeMeta[] = $state([]);
  let searchRequestId = 0;
  const unlisteners: Array<() => void> = [];
  let pendingScrollFrame: number | null = null;
  const ONBOARDING_SEEN_KEY = "vanta.onboarding.v2_3_seen";
  const QUICK_TIPS_HIDDEN_KEY = "vanta.quick_tips.hidden";
  let onboardingOpen = $state(false);
  let quickTipsVisible = $state(false);
  let pendingConfirmResult = $state<SearchResult | null>(null);
  let fadeDuration = $derived(vantaConfig?.accessibility?.reduced_motion ? 0 : 150);

  function scheduleScrollToSelected() {
    if (pendingScrollFrame !== null) return;
    pendingScrollFrame = requestAnimationFrame(() => {
      pendingScrollFrame = null;
      resultsListRef?.scrollToSelected();
    });
  }

  

  const builtinCommands: SearchResult[] = [
    {
      title: "Sleep",
      subtitle: "Suspend this machine immediately",
      icon: "fa-solid fa-moon",
      exec: "system-action:sleep",
      score: 1_400_000,
      match_indices: [],
      source: "Application",
      id: "cmd-sleep",
      section: "Commands",
    },
    {
      title: "Lock",
      subtitle: "Lock the current session",
      icon: "fa-solid fa-lock",
      exec: "system-action:lock",
      score: 1_390_000,
      match_indices: [],
      source: "Application",
      id: "cmd-lock",
      section: "Commands",
    },
    {
      title: "Shutdown",
      subtitle: "Power off this device",
      icon: "fa-solid fa-power-off",
      exec: "system-action:shutdown",
      score: 1_380_000,
      match_indices: [],
      source: "Application",
      id: "cmd-shutdown",
      section: "Commands",
    },
    {
      title: "Restart",
      subtitle: "Reboot the machine",
      icon: "fa-solid fa-rotate-right",
      exec: "system-action:restart",
      score: 1_370_000,
      match_indices: [],
      source: "Application",
      id: "cmd-restart",
      section: "Commands",
    },
    {
      title: "Go to BIOS",
      subtitle: "Reboot into firmware setup",
      icon: "fa-solid fa-microchip",
      exec: "system-action:bios",
      score: 1_350_000,
      match_indices: [],
      source: "Application",
      id: "cmd-bios",
      section: "Commands",
    },
    {
      title: "Log Out",
      subtitle: "Terminate the current session",
      icon: "fa-solid fa-right-from-bracket",
      exec: "system-action:logout",
      score: 1_360_000,
      match_indices: [],
      source: "Application",
      id: "cmd-logout",
      section: "Commands",
    },
  ];

  function filterCommandResults(q: string): SearchResult[] {
    const normalized = q.trim().toLowerCase();
    if (!normalized) return builtinCommands;

    return builtinCommands.filter((cmd) => {
      const inTitle = cmd.title.toLowerCase().includes(normalized);
      const inSubtitle = (cmd.subtitle || "").toLowerCase().includes(normalized);
      return inTitle || inSubtitle;
    });
  }

  function composeResults(base: SearchResult[], q: string): SearchResult[] {
    const commands = filterCommandResults(q);
    const macros = buildMacroResults(q);
    return [...commands, ...macros, ...base];
  }

  onDestroy(() => {
    if (pendingScrollFrame !== null) {
      cancelAnimationFrame(pendingScrollFrame);
      pendingScrollFrame = null;
    }
    for (const unlisten of unlisteners) {
      try {
        unlisten();
      } catch {
        // no-op
      }
    }
  });

  function injectThemeCss(css: string) {
    let styleEl = document.getElementById(
      "vanta-theme",
    ) as HTMLStyleElement | null;
    if (!styleEl) {
      styleEl = document.createElement("style");
      styleEl.id = "vanta-theme";
      document.head.appendChild(styleEl);
    }
    styleEl.textContent = css;
  }

  // Derived: total item count for navigation
  let totalItems = $derived(visibleRowCount);

  $effect(() => {
    if (extensionView || currentMode !== "launcher") return;
    const extVersion = availableExtensions.length;
    results = composeResults(baseResults, query);
    void extVersion;
  });

  let clipboardHistory: any[] = [];

  $effect(() => {
    const count = visibleRowCount;
    if (count === 0) {
      selectedIndex = 0;
      return;
    }
    if (selectedIndex >= count) {
      selectedIndex = count - 1;
    }
  });

  onMount(async () => {
    onboardingOpen = localStorage.getItem(ONBOARDING_SEEN_KEY) !== "1";
    quickTipsVisible = localStorage.getItem(QUICK_TIPS_HIDDEN_KEY) !== "1";
    isMiniPlayer =
      getCurrentWebviewWindow().label === "spotify-mini" ||
      window.location.search.includes("view=mini-player");

    // Register clipboard listener ASAP so startup emits aren't missed.
    try {
      unlisteners.push(
        await listen("open_clipboard", async () => {
          currentMode = "clipboard";
          query = "";
          try {
            clipboardHistory = await invoke("get_clipboard_history");
            updateClipboardSuggestions("");
          } catch (e) {
            console.error("Failed to fetch history", e);
          }
          await invoke("show_window");
          requestAnimationFrame(() => {
            requestAnimationFrame(() => searchInputRef?.focus?.());
          });
        }),
      );
    } catch (e) {
      console.error("Failed to bind clipboard listener", e);
    }

    try {
      availableThemes = await invoke<ThemeMeta[]>("get_installed_themes");
    } catch (e) {
      console.error("Failed to load themes:", e);
    }

    try {
      const config = await invoke<VantaConfig>("get_config");
      vantaConfig = config;
      applyTheme(config);

      const themeId = config.appearance.theme || "default";
      const targetTheme =
        availableThemes.find((t) => t.id === themeId) ||
        availableThemes.find((t) => t.id === "default");
      if (targetTheme) {
        injectThemeCss(targetTheme.css_content);
      }
    } catch (e) {
      console.error("Failed to load config:", e);
    }

    if (isMiniPlayer) {
      return;
    }

    try {
      availableExtensions = await invoke<ExtensionEntry[]>("get_extensions");
    } catch (e) {
      console.error("Failed to load extensions:", e);
    }

    try {
      availableMacros = await invoke<WorkflowMacro[]>("get_workflows");
    } catch (e) {
      console.error("Failed to load macros:", e);
    }

    try {
      macroJobs = await invoke<MacroJobRecord[]>("list_macro_jobs");
    } catch (e) {
      console.error("Failed to load macro jobs:", e);
    }

    // Listen for config hot-reload events
    unlisteners.push(
      await listen<VantaConfig>("config-updated", (event) => {
        console.log("Config updated:", event.payload);
        vantaConfig = event.payload;
        applyTheme(event.payload);

        const themeId = event.payload.appearance.theme || "default";
        const targetTheme =
          availableThemes.find((t) => t.id === themeId) ||
          availableThemes.find((t) => t.id === "default");
        if (targetTheme) {
          injectThemeCss(targetTheme.css_content);
        }
      }),
    );

    // Listen for blur status
    unlisteners.push(
      await listen<BlurStatus>("blur-status", (event) => {
        blurMode = event.payload.mode;
      }),
    );

    unlisteners.push(
      await listen<ExtensionEntry[]>("extensions-changed", (event) => {
        availableExtensions = event.payload;
        if (currentMode === "launcher") {
          if (query.trim() === "") {
            loadSuggestions();
          } else {
            handleSearch(query);
          }
        }
      }),
    );


    // Listen for app cache refresh events and update suggestions when idle.
    unlisteners.push(
      await listen("apps-changed", () => {
        if (currentMode === "launcher") {
          if (query.trim() === "") {
            loadSuggestions();
          } else {
            handleSearch(query);
          }
        }
      }),
    );

    unlisteners.push(
      await listen<WorkflowMacro[]>("macros-changed", (event) => {
        availableMacros = event.payload;
      }),
    );

    unlisteners.push(
      await listen<MacroJobRecord[]>("macro-jobs-updated", (event) => {
        macroJobs = event.payload;
      }),
    );

    // ── Click-away dismiss ──
    const appWindow = getCurrentWebviewWindow();
    unlisteners.push(
      await appWindow.onFocusChanged(({ payload: focused }) => {
        // Focus logic placeholder
      }),
    );

    // When the window is shown (hotkey/toggle/open), refresh apps once and refocus
    unlisteners.push(
      await listen("tauri://show", () => {
        isVisible = true;
        maybeRescanApps();
        requestAnimationFrame(() => {
          searchInputRef?.focus?.();
          if (query.trim() === "" && currentMode === "launcher") {
            loadSuggestions();
          }
        });
      }),
    );

    loadSuggestions();
    // Ensure app cache is fresh when GUI is first opened
    maybeRescanApps(true);
  });

  function updateClipboardSuggestions(q: string) {
    if (!q) {
      results = clipboardHistory.map((item) => ({
        title: item.content.replace(/\n/g, " ").substring(0, 100),
        subtitle: new Date(item.timestamp).toLocaleString(),
        icon: null,
        exec: `copy:${item.content}`,
        score: 0,
        source: "Clipboard",
        id: item.id,
        match_indices: [],
        section: "Clipboard",
      }));
    } else {
      const lower = q.toLowerCase();
      const filtered = clipboardHistory.filter((item) =>
        item.content.toLowerCase().includes(lower),
      );
      results = filtered.map((item) => ({
        title: item.content.replace(/\n/g, " ").substring(0, 100),
        subtitle: new Date(item.timestamp).toLocaleString(),
        icon: null,
        exec: `copy:${item.content}`,
        score: 0,
        source: "Clipboard",
        id: item.id,
        match_indices: [],
        section: "Clipboard",
      }));
    }
    selectedIndex = 0;
  }

  function buildMacroResults(q: string): SearchResult[] {
    if (!availableMacros.length) return [];
    const needle = q.trim().toLowerCase();
    return availableMacros
      .filter((macro) => {
        if (!macro.enabled) return false;
        if (!needle) return true;
        return (
          macro.id.toLowerCase().includes(needle) ||
          macro.name.toLowerCase().includes(needle) ||
          (macro.description ?? "").toLowerCase().includes(needle)
        );
      })
      .map((macro, idx) => ({
        title: macro.name,
        subtitle: macro.description ?? macro.id,
        icon: "fa-solid fa-diagram-project",
        exec: `macro:${macro.id}`,
        score: 1_300_000 - idx,
        match_indices: [],
        source: "Application",
        id: `macro-${macro.id}`,
        section: "Macros",
      }));
  }

  function parsePermissionError(err: unknown):
    | { type: "needed"; payload: PermissionNeededPayload }
    | null {
    const raw =
      typeof err === "string"
        ? err
        : typeof err === "object" && err && "message" in err
          ? String((err as any).message)
          : String(err ?? "");

    if (!raw) return null;

    const neededPrefix = "PERMISSION_NEEDED:";
    if (raw.startsWith(neededPrefix)) {
      try {
        const payload = JSON.parse(raw.slice(neededPrefix.length));
        return { type: "needed", payload } as any;
      } catch (e) {
        console.warn("Failed to parse PERMISSION_NEEDED payload", e, raw);
      }
    }

    return null;
  }

  function commandToExec(command: CommandContract): string {
    switch (command.kind) {
      case "launch_app":
        return command.exec;
      case "open_file":
        return command.path;
      case "open_settings":
        return "open-settings";
      case "open_store":
        return "open-store";
      case "copy_text":
        return `copy:${command.value}`;
      case "copy_path":
        return `copy-path:${command.value}`;
      case "reveal_path":
        return `reveal:${command.path}`;
      case "open_with_editor":
        return `open-with:${command.path}`;
      case "focus_window":
        return `focus:${command.id}`;
      case "close_window":
        return `close-window:${command.id}`;
      case "minimize_window":
        return `minimize-window:${command.id}`;
      case "move_window_current_workspace":
        return `move-window-current:${command.id}`;
      case "system_action":
        return `system-action:${command.action}`;
      case "macro_open":
        return `macro:${command.id}`;
      case "extension_view":
        return `ext-view:${command.ext_id}:${command.command}`;
      case "extension_action":
        return `ext-no-view:${command.ext_id}:${command.command}`;
      case "query_fill":
        return `fill:${command.value}`;
      case "profile_switch":
        return `profile-switch:${command.id}`;
      case "unknown":
        return command.exec;
    }
  }

  function inferCommandFromResult(result: SearchResult): CommandContract {
    const exec = result.exec;
    if (exec.startsWith("system-action:")) {
      return { kind: "system_action", action: exec.slice(14) };
    }
    if (exec === "open-settings") return { kind: "open_settings" };
    if (exec === "open-store") return { kind: "open_store" };
    if (exec.startsWith("copy:")) return { kind: "copy_text", value: exec.slice(5) };
    if (exec.startsWith("copy-path:")) return { kind: "copy_path", value: exec.slice(10) };
    if (exec.startsWith("reveal:")) return { kind: "reveal_path", path: exec.slice(7) };
    if (exec.startsWith("open-with:")) {
      return { kind: "open_with_editor", path: exec.slice(10) };
    }
    if (exec.startsWith("focus:")) return { kind: "focus_window", id: exec.slice(6) };
    if (exec.startsWith("close-window:")) {
      return { kind: "close_window", id: exec.slice(13) };
    }
    if (exec.startsWith("minimize-window:")) {
      return { kind: "minimize_window", id: exec.slice(16) };
    }
    if (exec.startsWith("move-window-current:")) {
      return { kind: "move_window_current_workspace", id: exec.slice(20) };
    }
    if (exec.startsWith("fill:")) return { kind: "query_fill", value: exec.slice(5) };
    if (exec.startsWith("profile-switch:")) {
      return { kind: "profile_switch", id: exec.slice(15) };
    }
    if (exec.startsWith("macro:")) return { kind: "macro_open", id: exec.slice(6) };
    if (exec.startsWith("ext-view:")) {
      const [ext_id, ...rest] = exec.slice(9).split(":");
      return { kind: "extension_view", ext_id, command: rest.join(":") };
    }
    if (exec.startsWith("ext-no-view:")) {
      const [ext_id, ...rest] = exec.slice(12).split(":");
      return { kind: "extension_action", ext_id, command: rest.join(":") };
    }
    if (result.source === "File") return { kind: "open_file", path: exec };
    return { kind: "launch_app", exec };
  }

  function commandForResult(result: SearchResult): CommandContract {
    return result.command ?? inferCommandFromResult(result);
  }

  function execForAction(result: SearchResult, action: { exec?: string; command?: CommandContract }): string {
    if (action.exec) return action.exec;
    if (action.command) return commandToExec(action.command);
    return result.exec;
  }

  function handlePermissionError(err: unknown) {
    const parsed = parsePermissionError(err);
    if (!parsed) return false;

    if (parsed.type === "needed") {
      const missingCaps = parsed.payload.missing_caps || [];
      const requestedCaps = parsed.payload.requested_caps || missingCaps;

      permissionPrompt = {
        scriptId: parsed.payload.script_id,
        missingCaps,
        requestedCaps,
      };
      return true;
    }

    return false;
  }

  function resetMacroState() {
    activeMacroId = null;
    macroArgs = {};
    macroDryRun = null;
    macroBusy = false;
    macroError = null;
  }

  async function dryRunMacro(macro: WorkflowMacro) {
    macroBusy = true;
    macroError = null;
    try {
      const result = await invoke<MacroDryRunResult>("dry_run_macro", {
        macroId: macro.id,
        args: macroArgs,
      });
      macroDryRun = result;
    } catch (e) {
      macroError = String(e);
      console.error("Dry-run failed", e);
    } finally {
      macroBusy = false;
    }
  }

  async function runMacro(macro: WorkflowMacro) {
    macroBusy = true;
    macroError = null;
    try {
      await invoke("start_macro_job", { macroId: macro.id, args: macroArgs });
      resetMacroState();
      resetAndHide();
    } catch (e) {
      if (handlePermissionError(e)) {
        macroBusy = false;
        return;
      }
      macroError = String(e);
      console.error("Run macro failed", e);
    } finally {
      macroBusy = false;
    }
  }

  async function retryMacroJob(jobId: string) {
    try {
      await invoke("retry_macro_job", { jobId });
    } catch (e) {
      console.error("Retry job failed", e);
    }
  }

  async function cancelMacroJob(jobId: string) {
    try {
      await invoke("cancel_macro_job", { jobId });
    } catch (e) {
      console.error("Cancel job failed", e);
    }
  }

  async function setDecision(
    decision: "Allow" | "Deny" | "Ask",
    caps: Capability[],
    note?: string,
  ) {
    if (!permissionPrompt) throw new Error("No permission prompt active");
    await invoke("set_permission_decision", {
      scriptId: permissionPrompt.scriptId,
      requestedCaps: caps,
      decision,
      note,
    });
  }

  async function handleAllowOnce() {
    if (!permissionPrompt) return;
    permissionBusy = true;
    const caps =
      permissionPrompt.requestedCaps.length
        ? permissionPrompt.requestedCaps
        : permissionPrompt.missingCaps;

    try {
      await setDecision("Allow", caps, "allow-once");
      await setDecision("Ask", caps, "reset-after-once");
      permissionPrompt = null;
    } catch (e) {
      console.error("Allow once failed", e);
    } finally {
      permissionBusy = false;
    }
  }

  async function handleAllowAlways() {
    if (!permissionPrompt) return;
    permissionBusy = true;
    const caps =
      permissionPrompt.requestedCaps.length
        ? permissionPrompt.requestedCaps
        : permissionPrompt.missingCaps;

    try {
      await setDecision("Allow", caps, "persist");
      permissionPrompt = null;
    } catch (e) {
      console.error("Allow always failed", e);
    } finally {
      permissionBusy = false;
    }
  }

  async function handleDeny() {
    if (!permissionPrompt) return;
    permissionBusy = true;
    const caps =
      permissionPrompt.requestedCaps.length
        ? permissionPrompt.requestedCaps
        : permissionPrompt.missingCaps;

    try {
      await setDecision("Deny", caps, "user-deny");
    } catch (e) {
      console.error("Deny failed", e);
    } finally {
      permissionPrompt = null;
      permissionBusy = false;
    }
  }

  async function loadSuggestions() {
    const requestId = ++searchRequestId;
    try {
      const suggestions = await invoke<SearchResult[]>("get_suggestions_v3");
      if (requestId !== searchRequestId) return;
      console.info("suggestions", suggestions?.length ?? 0);
      if (suggestions.length === 0) {
        // Fallback to a real search to avoid blank UI if suggestions fail.
        const searchFallback = await invoke<SearchResult[]>("search_v3", { query: "" });
        if (requestId !== searchRequestId) return;
        baseResults = searchFallback;
      } else {
        baseResults = suggestions;
      }

      results = composeResults(baseResults, "");

      if (results.length > 0) {
        selectedIndex = 1;
      }
    } catch (e) {
      console.error("Failed to load suggestions:", e);
      // Fallback to empty search results to keep UI populated
      try {
        const searchFallback = await invoke<SearchResult[]>("search_v3", { query: "" });
        if (requestId === searchRequestId) {
          baseResults = searchFallback;
          results = composeResults(baseResults, "");
          console.info("fallback search", results?.length ?? 0);
          if (results.length > 0) selectedIndex = 1;
        }
      } catch (err) {
        console.error("Fallback search failed:", err);
      }
    }
  }

  async function handleSearch(q: string) {
    const requestId = ++searchRequestId;
    if (currentMode === "clipboard") {
      updateClipboardSuggestions(q);
      return;
    }

    if (!q.trim()) {
      resetMacroState();
      searchTime = null;
      await loadSuggestions();
      return;
    }

    resetMacroState();

    try {
      const start = performance.now();
      const searchResults = await invoke<SearchResult[]>("search_v3", {
        query: q,
      });
      if (requestId !== searchRequestId) return;
      const elapsed = performance.now() - start;

      console.info("search results", searchResults?.length ?? 0, "for", q);
      baseResults = searchResults;
      results = composeResults(baseResults, q);
      selectedIndex = results.length > 0 ? 1 : 0;
      searchTime = elapsed;
    } catch (e) {
      if (requestId !== searchRequestId) return;
      console.error("Search failed:", e);
      baseResults = [];
      results = [];
      searchTime = null;
    }
  }

  async function saveConfigUpdate(mutator: (cfg: VantaConfig) => void) {
    if (!vantaConfig) return;
    mutator(vantaConfig);
    applyTheme(vantaConfig);

    const themeId = vantaConfig.appearance.theme || "default";
    const targetTheme =
      availableThemes.find((t) => t.id === themeId) ||
      availableThemes.find((t) => t.id === "default");
    if (targetTheme) {
      injectThemeCss(targetTheme.css_content);
    }

    try {
      await invoke("save_config", { newConfig: vantaConfig });
    } catch (e) {
      console.error("Failed to save onboarding update", e);
    }
  }

  async function applyOnboardingHotkey(hotkey: string) {
    await saveConfigUpdate((cfg) => {
      cfg.general.hotkey = hotkey;
    });
  }

  async function applyOnboardingTheme(themeId: string) {
    await saveConfigUpdate((cfg) => {
      cfg.appearance.theme = themeId;
    });
  }

  async function applyOnboardingIncludeHidden(includeHidden: boolean) {
    await saveConfigUpdate((cfg) => {
      cfg.files.include_hidden = includeHidden;
    });
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
    query = text;
    void handleSearch(text);
    requestAnimationFrame(() => searchInputRef?.focus?.());
  }

  function getConfirmSpec(result: SearchResult): { title: string; description: string; confirmLabel: string } | null {
    const command = commandForResult(result);
    if (command.kind === "close_window") {
      return {
        title: "Close Window?",
        description: `This will close \"${result.title}\" immediately.`,
        confirmLabel: "Close",
      };
    }

    if (command.kind !== "system_action") return null;

    const action = command.action;
    if (action === "shutdown") {
      return {
        title: "Shut Down System?",
        description: "This powers off the device immediately.",
        confirmLabel: "Shut Down",
      };
    }

    if (action === "restart") {
      return {
        title: "Restart System?",
        description: "This reboots the device and closes active sessions.",
        confirmLabel: "Restart",
      };
    }

    if (action === "sleep") {
      return {
        title: "Sleep System?",
        description: "This suspends the current session.",
        confirmLabel: "Sleep",
      };
    }

    if (action === "logout") {
      return {
        title: "Log Out Session?",
        description: "This terminates the current user session.",
        confirmLabel: "Log Out",
      };
    }

    if (action === "bios") {
      return {
        title: "Reboot To Firmware Setup?",
        description: "This restarts the machine and enters firmware setup if supported.",
        confirmLabel: "Reboot",
      };
    }

    return null;
  }

  async function handleActivate(result: SearchResult, bypassConfirm = false) {
    if (permissionPrompt) return; // modal blocks background actions
    if (!bypassConfirm) {
      const confirm = getConfirmSpec(result);
      if (confirm) {
        pendingConfirmResult = result;
        return;
      }
    }

    if (actionInFlight) return;
    actionInFlight = true;
    try {
      const command = commandForResult(result);
      if (command.kind === "macro_open") {
        const id = command.id;
        const macro = availableMacros.find((m) => m.id === id);
        if (macro) {
          activeMacroId = id;
          macroArgs = Object.fromEntries(
            (macro.args ?? []).map((a) => [a.name, a.default_value ?? ""]),
          );
          macroDryRun = null;
          macroError = null;
        }
        return;
      }
      if (command.kind === "extension_view") {
        const extId = command.ext_id;
        const cmd = command.command;
        const ext = availableExtensions.find((e) => e.manifest.name === extId);
        extensionView = { extId, command: cmd, extPath: ext?.path ?? "" };
        return;
      } else if (command.kind === "extension_action") {
        const extId = command.ext_id;
        const cmd = command.command;
        const ext = availableExtensions.find((e) => e.manifest.name === extId);
        extensionView = { extId, command: cmd, extPath: ext?.path ?? "" };
        return;
      } else if (command.kind === "query_fill") {
        query = command.value;
        await handleSearch(query);
        requestAnimationFrame(() => {
          requestAnimationFrame(() => searchInputRef?.focus?.());
        });
        return;
      } else if (command.kind === "profile_switch") {
        try {
          const updated = await invoke<VantaConfig>("switch_profile", {
            profileId: command.id,
          });
          vantaConfig = updated;
          applyTheme(updated);
          const themeId = updated.appearance.theme || "default";
          const targetTheme =
            availableThemes.find((t) => t.id === themeId) ||
            availableThemes.find((t) => t.id === "default");
          if (targetTheme) {
            injectThemeCss(targetTheme.css_content);
          }
          await loadSuggestions();
          searchInputRef?.focus?.();
        } catch (e) {
          console.error("Profile switch failed", e);
        }
        return;
      } else if (command.kind === "system_action") {
        const action = command.action;
        try {
          await invoke("system_action", { action });
          resetAndHide();
        } catch (e) {
          console.error("System action failed", e);
        }
        return;
      } else if (command.kind === "open_settings") {
        view = "settings";
        return;
      } else if (command.kind === "open_store") {
        view = "store";
        return;
      } else if (command.kind === "copy_path") {
        const value = command.value;
        await navigator.clipboard.writeText(value);
        resetAndHide();
      } else if (command.kind === "reveal_path") {
        const target = command.path;
        await invoke("reveal_in_file_manager", { path: target });
        resetAndHide();
      } else if (command.kind === "open_with_editor") {
        const target = command.path;
        await invoke("open_with_editor", { path: target });
        resetAndHide();
      } else if (command.kind === "copy_text") {
        const value = command.value;
        await navigator.clipboard.writeText(value);
        resetAndHide();
      } else if (command.kind === "open_file") {
        await invoke("open_path", { path: command.path });
        resetAndHide();
      } else {
        await invoke("launch_app", { exec: commandToExec(command) });
        resetAndHide();
      }
    } catch (e) {
      console.error("Launch/Copy failed:", e);
    } finally {
      actionInFlight = false;
    }
  }

  function resetAndHide() {
    query = "";
    loadSuggestions();

    extensionView = null;
    view = "launcher";
    currentMode = "launcher";
    resetMacroState();
    selectedIndex = 0;
    searchTime = null;
    isVisible = false;
    invoke("hide_window").catch((e) => console.error("Hide failed:", e));
  }

  async function handleEscape() {
    if (permissionPrompt) return;
    if (view === "store") {
      view = "launcher";
      return;
    }
    if (extensionView) {
      extensionView = null;
      return;
    }
    if (activeMacroId) {
      resetMacroState();
      return;
    }
    if (currentMode === "clipboard" && query === "") {
      resetAndHide();
      return;
    }
    resetAndHide();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (pendingConfirmResult) {
      if (e.key === "Escape") {
        e.preventDefault();
        pendingConfirmResult = null;
      }
      return;
    }

    if (onboardingOpen) return;
    if (permissionPrompt) {
      return; // modal owns keyboard focus
    }
    if (activeMacroId && e.key === "Escape") {
      resetMacroState();
      return;
    }
    // Global: Ctrl+, toggles settings
    if (e.key === "," && e.ctrlKey) {
      e.preventDefault();
      view = view === "launcher" ? "settings" : "launcher";
      return;
    }

    if (view === "settings" || view === "store") {
      if (e.key === "Escape") {
        e.preventDefault();
        e.stopPropagation();
        view = "launcher";
      }
      return;
    }

    // Ensure search input gets focus if the user types anything
    // Ignore modifiers (ctrl/meta/alt) unless it's just a single character
    if (!e.ctrlKey && !e.metaKey && !e.altKey && e.key.length === 1) {
      if (document.activeElement?.tagName !== "INPUT") {
        searchInputRef?.focus();
      }
    }

    if (e.key === "Backspace" && document.activeElement?.tagName !== "INPUT") {
      searchInputRef?.focus();
    }

    if (totalItems === 0) return;

    const activeRow = resultsListRef?.getVisibleRow(selectedIndex);
    const activeResult =
      activeRow && activeRow.type === "item" ? activeRow.result : undefined;

    if (activeResult && activeResult.actions?.length) {
      const findAction = (prefix: string) =>
        activeResult.actions?.find((a) => execForAction(activeResult, a).startsWith(prefix));

      if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "c") {
        const action = findAction("copy-path:");
        if (action) {
          e.preventDefault();
          handleActivate({
            ...activeResult,
            exec: execForAction(activeResult, action),
            command: action.command,
          });
          return;
        }
      }

      if (e.key === "Enter" && e.shiftKey) {
        const action = findAction("reveal:") || activeResult.actions[0];
        if (action) {
          e.preventDefault();
          handleActivate({
            ...activeResult,
            exec: execForAction(activeResult, action),
            command: action.command,
          });
          return;
        }
      }

      if (e.key === "Enter" && e.altKey) {
        const action = findAction("open-with:") || activeResult.actions[0];
        if (action) {
          e.preventDefault();
          handleActivate({
            ...activeResult,
            exec: execForAction(activeResult, action),
            command: action.command,
          });
          return;
        }
      }

      if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "m") {
        const action = findAction("move-window-current:");
        if (action) {
          e.preventDefault();
          handleActivate({
            ...activeResult,
            exec: execForAction(activeResult, action),
            command: action.command,
          });
          return;
        }
      }

      if (e.ctrlKey && !e.shiftKey && e.key.toLowerCase() === "m") {
        const action = findAction("minimize-window:");
        if (action) {
          e.preventDefault();
          handleActivate({
            ...activeResult,
            exec: execForAction(activeResult, action),
            command: action.command,
          });
          return;
        }
      }
    }

    function skipHeaders(idx: number, direction: 1 | -1): number {
      let i = idx;
      let safety = totalItems;
      while (safety-- > 0 && resultsListRef?.getVisibleRow(i)?.type === "header") {
        i = direction === 1
          ? (i + 1) % totalItems
          : (i <= 0 ? totalItems - 1 : i - 1);
      }
      return i;
    }

    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        selectedIndex = skipHeaders((selectedIndex + 1) % totalItems, 1);
        scheduleScrollToSelected();
        break;
      case "ArrowUp":
        e.preventDefault();
        selectedIndex = skipHeaders(
          selectedIndex <= 0 ? totalItems - 1 : selectedIndex - 1, -1
        );
        scheduleScrollToSelected();
        break;
      case "Enter":
        e.preventDefault();
        if (activeResult) {
          handleActivate(activeResult);
        }
        break;
      case "Tab":
        e.preventDefault();
        if (results[selectedIndex]) {
          const res = results[selectedIndex];
          if (res.exec.startsWith("fill:")) {
            handleActivate(res);
            break;
          }
        }

        if (e.shiftKey) {
          selectedIndex = skipHeaders(
            selectedIndex <= 0 ? totalItems - 1 : selectedIndex - 1, -1
          );
        } else {
          selectedIndex = skipHeaders((selectedIndex + 1) % totalItems, 1);
        }

        scheduleScrollToSelected();
        break;
    }
  }
</script>

<svelte:window
  onkeydown={handleKeydown}
  onfocus={() => {
    if (view !== "settings") searchInputRef?.focus();
  }}
/>

{#if isMiniPlayer}
  <SpotifyMiniPlayer />
{:else}
<div
  class="vanta-root vanta-glass"
  class:css-blur={blurMode === "fallback"}
  role="application"
>
  {#if view === "store"}
    <div
      in:fade={{ duration: fadeDuration }}
      style="height: 100%; width: 100%; position: relative;"
    >
      <StoreView onClose={() => (view = "launcher")} />
    </div>
  {:else if view === "settings" && vantaConfig}
    <div
      in:fade={{ duration: fadeDuration }}
      style="height: 100%; width: 100%; position: relative;"
    >
      <SettingsView
        bind:config={vantaConfig}
        {availableThemes}
        onClose={() => { view = "launcher"; loadSuggestions(); }}
      />
    </div>
  {:else if currentMode === "clipboard"}
    <div
      in:fade={{ duration: fadeDuration }}
      style="height: 100%; width: 100%;"
    >
      <ClipboardView
        onEscape={handleEscape}
      />
    </div>
  {:else if extensionView}
    <div
      in:fade={{ duration: fadeDuration }}
      style="height: 100%; width: 100%;"
    >
      <ExtensionHost
        extId={extensionView.extId}
        commandName={extensionView.command}
        extPath={extensionView.extPath}
        onClose={() => { extensionView = null; }}
        onToast={(opts) => console.log('[toast]', opts.title, opts.message)}
      />
    </div>
  {:else}
    <div
      in:fade={{ duration: fadeDuration }}
      style="height: 100%; width: 100%; display: grid; grid-template-rows: auto 1fr auto;"
    >
      <SearchInput
        bind:this={searchInputRef}
        bind:query
        onSearch={handleSearch}
        onEscape={handleEscape}
      />

      {#if !onboardingOpen && quickTipsVisible && query.trim() === ""}
        <QuickTipsPanel
          onTry={applyTipQuery}
          onDismiss={dismissQuickTips}
        />
      {/if}

      {#if currentMacro}
        <MacroPreview
          macro={currentMacro}
          args={macroArgs}
          dryRun={macroDryRun}
          busy={macroBusy}
          error={macroError}
          onArgsChange={(name, value) => (macroArgs = { ...macroArgs, [name]: value })}
          onDryRun={() => dryRunMacro(currentMacro)}
          onRun={() => runMacro(currentMacro)}
          onClose={resetMacroState}
        />
      {:else if activeMacroId}
        <div class="empty-state">Macro not found</div>
      {:else}
        <ResultsList
          bind:this={resultsListRef}
          {results}
          bind:selectedIndex
          onActivate={handleActivate}
          on:visiblecount={(event) => (visibleRowCount = event.detail.count)}
        />

        {#if query.trim() !== "" && vantaConfig && vantaConfig.search.show_explain_panel !== false}
          <SearchExplainPanel
            query={query}
            results={results}
            searchConfig={vantaConfig.search}
          />
        {/if}
      {/if}

      <NowPlayingBar />
      <StatusBar
        resultCount={
          activeMacroId
            ? macroDryRun?.steps.length ?? 0
            : visibleRowCount
        }
        {searchTime}
      />

      {#if pendingConfirmResult}
        {@const confirmSpec = getConfirmSpec(pendingConfirmResult)}
        {#if confirmSpec}
          <ActionConfirmModal
            title={confirmSpec.title}
            description={confirmSpec.description}
            confirmLabel={confirmSpec.confirmLabel}
            onCancel={() => (pendingConfirmResult = null)}
            onConfirm={() => {
              const confirmed = pendingConfirmResult;
              pendingConfirmResult = null;
              if (confirmed) {
                void handleActivate(confirmed, true);
              }
            }}
          />
        {/if}
      {/if}

      {#if onboardingOpen && vantaConfig}
        <FirstRunWizard
          hotkey={vantaConfig.general.hotkey}
          themeId={vantaConfig.appearance.theme}
          includeHidden={vantaConfig.files.include_hidden}
          themes={availableThemes}
          on:tryquery={(event) => applyTipQuery(event.detail.query)}
          on:sethotkey={(event) => void applyOnboardingHotkey(event.detail.hotkey)}
          on:settheme={(event) => void applyOnboardingTheme(event.detail.themeId)}
          on:setincludehidden={(event) =>
            void applyOnboardingIncludeHidden(event.detail.includeHidden)}
          on:opensettings={() => {
            completeOnboarding();
            view = "settings";
          }}
          on:skip={completeOnboarding}
          on:complete={completeOnboarding}
        />

        {#if query.trim() === ""}
          <JobsPanel
            jobs={macroJobs}
            onRetry={retryMacroJob}
            onCancel={cancelMacroJob}
          />
        {/if}
      {/if}
    </div>
  {/if}

  {#if permissionPrompt}
    <PermissionModal
      scriptId={permissionPrompt.scriptId}
      missingCaps={permissionPrompt.missingCaps}
      onAllowOnce={handleAllowOnce}
      onAllowAlways={handleAllowAlways}
      onDeny={handleDeny}
      busy={permissionBusy}
    />
  {/if}
</div>
{/if}

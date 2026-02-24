<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { onDestroy, onMount } from "svelte";
  import { fade } from "svelte/transition";
  import type {
    VantaConfig,
    SearchResult,
    BlurStatus,
    ScriptEntry,
    ScriptItem,
    ScriptOutput,
    ThemeMeta,
    Capability,
    PermissionNeededPayload,
    PermissionDeniedPayload,
  } from "$lib/types";
  import { applyTheme } from "$lib/theme";
  import SearchInput from "$lib/components/SearchInput.svelte";
  import ResultsList from "$lib/components/ResultsList.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  import ScriptResultItem from "$lib/components/ScriptResultItem.svelte";
  import SettingsView from "$lib/components/SettingsView.svelte";
  import PermissionModal from "$lib/components/PermissionModal.svelte";

  let query = $state("");
  let vantaConfig: VantaConfig | undefined = $state();
  let results: SearchResult[] = $state([]);
  let baseResults: SearchResult[] = $state([]);
  let view: "launcher" | "settings" = $state("launcher");
  let scriptResults: ScriptItem[] = $state([]);
  let isScriptMode = $state(false);
  let activeScriptKeyword = $state("");
  let selectedIndex = $state(0);
  let searchTime: number | null = $state(null);
  let visibleRowCount = $state(0);
    let currentMode: "launcher" | "clipboard" = $state("launcher");
  let preferDefaultOrder = $derived(
    query.trim() === "" && currentMode === "launcher" && !isScriptMode,
  );
  let blurMode: string = $state("fallback");
  let searchInputRef: SearchInput | undefined = $state();
  let resultsListRef: ResultsList | undefined = $state();
  let availableScripts: ScriptEntry[] = $state([]);
  let actionInFlight = $state(false); // guard: don't dismiss during actions
  let isVisible = $state(true);
  let isMouseOver = $state(false);
  let permissionPrompt = $state<{
    scriptId: string;
    missingCaps: Capability[];
    requestedCaps: Capability[];
    keyword: string;
    args: string;
  } | null>(null);
  let permissionBusy = $state(false);

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

  let activeScriptCaps = $derived(
    activeScriptKeyword ? capsForScript(activeScriptKeyword) : [],
  );

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

  function buildScriptResults(q: string): SearchResult[] {
    if (!availableScripts.length) return [];
    const normalized = q.trim().toLowerCase();

    return availableScripts
      .filter((script) => {
        if (!normalized) return true;
        const name = script.name || script.keyword;
        const desc = script.description || "";
        return (
          script.keyword.toLowerCase().includes(normalized) ||
          name.toLowerCase().includes(normalized) ||
          desc.toLowerCase().includes(normalized)
        );
      })
      .map((script, index) => ({
        title: script.name || script.keyword,
        subtitle: script.description || script.path,
        icon: script.icon || null,
        exec: `run-script:${script.keyword}`,
        score: 1_200_000 - index,
        match_indices: [],
        source: { Script: { keyword: script.keyword } },
        id: `script-${script.keyword}`,
        section: "Scripts",
      }));
  }

  function composeResults(base: SearchResult[], q: string): SearchResult[] {
    const commands = filterCommandResults(q);
    const scripts = buildScriptResults(q);
    return [...commands, ...base, ...scripts];
  }

  onDestroy(() => {
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
  let totalItems = $derived(
    isScriptMode ? scriptResults.length : visibleRowCount,
  );

  $effect(() => {
    if (isScriptMode || currentMode !== "launcher") return;
    const scriptsVersion = availableScripts.length;
    results = composeResults(baseResults, query);
    void scriptsVersion; // ensure reactivity when scripts change
  });

  // Clipboard State
  let clipboardHistory: any[] = [];

  // Keep selection valid when result sets change
  $effect(() => {
    const count = isScriptMode ? scriptResults.length : visibleRowCount;
    if (count === 0) {
      selectedIndex = 0;
      return;
    }
    if (selectedIndex >= count) {
      selectedIndex = count - 1;
    }
  });

  onMount(async () => {
    // ... existing onMount code ...

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
          setTimeout(() => searchInputRef?.focus?.(), 50);
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

    // Fetch available scripts
    try {
      availableScripts = await invoke<ScriptEntry[]>("get_scripts");
      console.log("Available scripts:", availableScripts);
    } catch (e) {
      console.error("Failed to load scripts:", e);
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

    // Listen for script changes (hot-reload)
    unlisteners.push(
      await listen<ScriptEntry[]>("scripts-changed", (event) => {
        availableScripts = event.payload;
        console.log("Scripts updated:", availableScripts.length);
      }),
    );

    // Refresh scripts after store installs finish
    unlisteners.push(
      await listen("download_status", (event) => {
        const status = (event.payload as any)?.status;
        if (status === "success") {
          refreshScripts();
        }
      }),
    );


    // Listen for app cache refresh events and update suggestions when idle.
    unlisteners.push(
      await listen("apps-changed", () => {
        if (currentMode === "launcher" && query.trim() === "") {
          loadSuggestions();
        }
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
        setTimeout(() => {
          searchInputRef?.focus?.();
          if (query.trim() === "" && currentMode === "launcher") {
            loadSuggestions();
          }
        }, 50);
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

  /**
            section: "Clipboard",
   * Check if query begins with a known script keyword.
   * Returns [keyword, args] if matched, or null.
   */
  function detectScriptQuery(q: string): [string, string] | null {
    const trimmed = q.trim();
    if (!trimmed) return null;

    const firstSpace = trimmed.indexOf(" ");
    const keyword = firstSpace > 0 ? trimmed.slice(0, firstSpace) : trimmed;
    const args = firstSpace > 0 ? trimmed.slice(firstSpace + 1).trim() : "";

    const script = availableScripts.find(
      (s) => s.keyword.toLowerCase() === keyword.toLowerCase(),
    );
    if (script) return [script.keyword, args];
    return null;
  }

  function capsForScript(keyword: string): Capability[] {
    const match = availableScripts.find(
      (s) => s.keyword.toLowerCase() === keyword.toLowerCase(),
    );
    return match?.capabilities ?? [];
  }

  function capIcon(cap: Capability): string {
    switch (cap) {
      case "Network":
        return "network";
      case "Shell":
        return "shell";
      case "Filesystem":
        return "filesystem";
    }
  }

  function parsePermissionError(err: unknown):
    | { type: "needed"; payload: PermissionNeededPayload }
    | { type: "denied"; payload: PermissionDeniedPayload }
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

    const deniedPrefix = "PERMISSION_DENIED:";
    if (raw.startsWith(deniedPrefix)) {
      try {
        const payload = JSON.parse(raw.slice(deniedPrefix.length));
        return { type: "denied", payload } as any;
      } catch (e) {
        console.warn("Failed to parse PERMISSION_DENIED payload", e, raw);
      }
    }

    return null;
  }

  function handlePermissionError(err: unknown, keyword: string, args: string) {
    const parsed = parsePermissionError(err);
    if (!parsed) return false;

    if (parsed.type === "needed") {
      const missingCaps = parsed.payload.missing_caps || [];
      const requestedCaps = parsed.payload.requested_caps || missingCaps;

      permissionPrompt = {
        scriptId: parsed.payload.script_id || keyword,
        missingCaps,
        requestedCaps,
        keyword,
        args,
      };
      // Keep the script view active with a clear placeholder
      isScriptMode = true;
      activeScriptKeyword = keyword;
      scriptResults = [
        {
          title: "Permission required",
          subtitle: `Grant access to run ${keyword}`,
          icon: "lock",
          urgency: "critical",
        },
      ];
      searchTime = null;
      return true;
    }

    if (parsed.type === "denied") {
      const caps = parsed.payload.requested_caps || [];
      isScriptMode = true;
      activeScriptKeyword = keyword;
      scriptResults = [
        {
          title: "Permission denied",
          subtitle: caps.length
            ? `Requested: ${caps.join(", ")}`
            : "Script denied by policy",
          icon: "lock",
          urgency: "critical",
        },
      ];
      searchTime = null;
      return true;
    }

    return false;
  }

  async function runScriptWithCaps(
    keyword: string,
    args: string,
    caps: Capability[],
  ) {
    const start = performance.now();
    try {
      const output = await invoke<ScriptOutput>("execute_script", {
        keyword,
        args,
        caps,
      });

      scriptResults = output.items;
      isScriptMode = true;
      activeScriptKeyword = keyword;
      selectedIndex = 0;
      searchTime = performance.now() - start;
      return true;
    } catch (e) {
      if (handlePermissionError(e, keyword, args)) return false;
      console.error("Script execution failed (retry)", e);
      scriptResults = [
        {
          title: `Script Error: ${keyword}`,
          subtitle: String(e),
          icon: "error",
          urgency: "critical",
        },
      ];
      searchTime = null;
      return false;
    }
  }

  async function refreshScripts() {
    try {
      availableScripts = await invoke<ScriptEntry[]>("get_scripts");
      if (!isScriptMode && currentMode === "launcher") {
        results = composeResults(baseResults, query);
      }
    } catch (e) {
      console.error("Failed to refresh scripts", e);
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
      const ran = await runScriptWithCaps(
        permissionPrompt.keyword,
        permissionPrompt.args,
        caps,
      );
      if (ran) {
        await setDecision("Ask", caps, "reset-after-once");
        permissionPrompt = null;
      }
    } catch (e) {
      console.error("Allow once failed", e);
      scriptResults = [
        {
          title: "Permission decision failed",
          subtitle: String(e),
          icon: "error",
          urgency: "critical",
        },
      ];
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
      const ran = await runScriptWithCaps(
        permissionPrompt.keyword,
        permissionPrompt.args,
        caps,
      );
      if (ran) {
        permissionPrompt = null;
      }
    } catch (e) {
      console.error("Allow always failed", e);
      scriptResults = [
        {
          title: "Permission decision failed",
          subtitle: String(e),
          icon: "error",
          urgency: "critical",
        },
      ];
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
      scriptResults = [
        {
          title: "Blocked",
          subtitle: "You denied this script's request.",
          icon: "lock",
          urgency: "critical",
        },
      ];
      searchTime = null;
    } catch (e) {
      console.error("Deny failed", e);
      scriptResults = [
        {
          title: "Permission decision failed",
          subtitle: String(e),
          icon: "error",
          urgency: "critical",
        },
      ];
    } finally {
      permissionPrompt = null;
      permissionBusy = false;
    }
  }

  async function loadSuggestions() {
    const requestId = ++searchRequestId;
    try {
      const suggestions = await invoke<SearchResult[]>("get_suggestions");
      if (requestId !== searchRequestId) return;
      console.info("suggestions", suggestions?.length ?? 0);
      if (suggestions.length === 0) {
        // Fallback to a real search to avoid blank UI if suggestions fail.
        const searchFallback = await invoke<SearchResult[]>("search", { query: "" });
        if (requestId !== searchRequestId) return;
        baseResults = searchFallback;
      } else {
        baseResults = suggestions;
      }

      results = composeResults(baseResults, "");

      if (results.length > 0) {
        selectedIndex = 0;
      }
    } catch (e) {
      console.error("Failed to load suggestions:", e);
      // Fallback to empty search results to keep UI populated
      try {
        const searchFallback = await invoke<SearchResult[]>("search", { query: "" });
        if (requestId === searchRequestId) {
          baseResults = searchFallback;
          results = composeResults(baseResults, "");
          console.info("fallback search", results?.length ?? 0);
          if (results.length > 0) selectedIndex = 0;
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
      isScriptMode = false;
      activeScriptKeyword = "";
      scriptResults = [];
      searchTime = null;
      // Load suggestions instead of clearing results
      await loadSuggestions();
      return;
    }

    // Check if query triggers a script
    const scriptMatch = detectScriptQuery(q);
    if (scriptMatch) {
      const [keyword, args] = scriptMatch;
      isScriptMode = true;
      activeScriptKeyword = keyword;
      baseResults = [];
      results = [];
      const caps = capsForScript(keyword);

      const localRequestId = requestId;
      const ran = await runScriptWithCaps(keyword, args, caps);
      if (localRequestId !== searchRequestId) return;
      if (ran) {
        selectedIndex = 0;
      }
      return;
    }

    // Normal app search
    isScriptMode = false;
    activeScriptKeyword = "";
    scriptResults = [];

    try {
      const start = performance.now();
      const searchResults = await invoke<SearchResult[]>("search", {
        query: q,
      });
      if (requestId !== searchRequestId) return;
      const elapsed = performance.now() - start;

      console.info("search results", searchResults?.length ?? 0, "for", q);
      baseResults = searchResults;
      results = composeResults(baseResults, q);
      selectedIndex = 0;
      searchTime = elapsed;
    } catch (e) {
      if (requestId !== searchRequestId) return;
      console.error("Search failed:", e);
      baseResults = [];
      results = [];
      searchTime = null;
    }
  }

  async function handleActivate(result: SearchResult) {
    if (permissionPrompt) return; // modal blocks background actions
    if (actionInFlight) return;
    actionInFlight = true;
    try {
      if (result.exec.startsWith("fill:")) {
        query = result.exec.slice(5);
        await handleSearch(query);
        setTimeout(() => {
          document.querySelector("input")?.focus();
        }, 10);
        return;
      } else if (result.exec.startsWith("run-script:")) {
        const keyword = result.exec.slice(11);
        const caps = capsForScript(keyword);
        await runScriptWithCaps(keyword, "", caps);
        return;
      } else if (result.exec.startsWith("system-action:")) {
        const action = result.exec.slice(14);
        try {
          await invoke("system_action", { action });
          resetAndHide();
        } catch (e) {
          console.error("System action failed", e);
        }
        return;
      } else if (result.exec === "open-settings") {
        view = "settings";
        return;
      } else if (result.exec.startsWith("copy-path:")) {
        const value = result.exec.slice(10);
        await navigator.clipboard.writeText(value);
        resetAndHide();
      } else if (result.exec.startsWith("reveal:")) {
        const target = result.exec.slice(7);
        await invoke("reveal_in_file_manager", { path: target });
        resetAndHide();
      } else if (result.exec.startsWith("open-with:")) {
        const target = result.exec.slice(10);
        await invoke("open_with_editor", { path: target });
        resetAndHide();
      } else if (result.exec === "doctor:run") {
        await invoke("run_doctor_terminal");
        resetAndHide();
      } else if (result.exec.startsWith("copy:")) {
        const value = result.exec.slice(5);
        await navigator.clipboard.writeText(value);
        resetAndHide();
      } else if (result.exec.startsWith("install:")) {
        const target = result.exec.slice(8);
        const ok = window.confirm(
          `Install this script or package?\n${target}`,
        );
        if (!ok) return;
        await invoke("launch_app", { exec: result.exec });
        refreshScripts();
        // deliberately leaving window open to view progress
      } else if (result.source === "File") {
        await invoke("open_path", { path: result.exec });
        resetAndHide();
      } else {
        await invoke("launch_app", { exec: result.exec });
        resetAndHide();
      }
    } catch (e) {
      console.error("Launch/Copy failed:", e);
    } finally {
      actionInFlight = false;
    }
  }

  async function handleScriptActivate(item: ScriptItem) {
    if (permissionPrompt) return; // modal blocks background actions
    if (actionInFlight) return;
    if (!item.action) return;

    actionInFlight = true;
    try {
      switch (item.action.type) {
        case "copy":
          await navigator.clipboard.writeText(item.action.value);
          resetAndHide();
          break;
        case "open":
          await invoke("plugin:opener|open_url", { url: item.action.value });
          resetAndHide();
          break;
        case "run":
          await invoke("launch_app", { exec: item.action.value });
          resetAndHide();
          break;
      }
    } catch (e) {
      console.error("Script action failed:", e);
    } finally {
      actionInFlight = false;
    }
  }

  /**
   * Reset all state and hide the window.
   */
  function resetAndHide() {
    query = "";
    // Don't clear results immediately, so next open has something.
    // Ideally we refresh them.
    loadSuggestions();

    scriptResults = [];
    isScriptMode = false;
    currentMode = "launcher"; // Reset mode
    activeScriptKeyword = "";
    selectedIndex = 0;
    searchTime = null;
    isVisible = false;
    invoke("hide_window").catch((e) => console.error("Hide failed:", e));
  }

  async function handleEscape() {
    if (permissionPrompt) return; // hard-block: modal must be resolved
    if (currentMode === "clipboard" && query === "") {
      // Close the window entirely instead of staying in launcher
      resetAndHide();
      return;
    }
    resetAndHide();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (permissionPrompt) {
      return; // modal owns keyboard focus
    }
    // Global: Ctrl+, toggles settings
    if (e.key === "," && e.ctrlKey) {
      e.preventDefault();
      view = view === "launcher" ? "settings" : "launcher";
      return;
    }

    // If in settings, intercept Esc/Enter to close
    if (view === "settings") {
      if (e.key === "Escape" || e.key === "Enter") {
        e.preventDefault();
        e.stopPropagation();
        view = "launcher";
      }
      return; // Block other keys
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

    // Launcher logic
    if (totalItems === 0) return;

    const activeRow = !isScriptMode
      ? resultsListRef?.getVisibleRow(selectedIndex)
      : undefined;
    const activeResult =
      activeRow && activeRow.type === "item" ? activeRow.result : undefined;

    // Secondary action shortcuts for normal results
    if (!isScriptMode && activeResult && activeResult.actions?.length) {
      const findAction = (prefix: string) =>
        activeResult.actions?.find((a) => a.exec.startsWith(prefix));

      if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "c") {
        const action = findAction("copy-path:");
        if (action) {
          e.preventDefault();
          handleActivate({ ...activeResult, exec: action.exec });
          return;
        }
      }

      if (e.key === "Enter" && e.shiftKey) {
        const action = findAction("reveal:") || activeResult.actions[0];
        if (action) {
          e.preventDefault();
          handleActivate({ ...activeResult, exec: action.exec });
          return;
        }
      }

      if (e.key === "Enter" && e.altKey) {
        const action = findAction("open-with:") || activeResult.actions[0];
        if (action) {
          e.preventDefault();
          handleActivate({ ...activeResult, exec: action.exec });
          return;
        }
      }
    }

    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        selectedIndex = (selectedIndex + 1) % totalItems;
        // Manual scroll trigger
        if (!isScriptMode)
          setTimeout(() => resultsListRef?.scrollToSelected(), 0);
        break;
      case "ArrowUp":
        e.preventDefault();
        selectedIndex = selectedIndex <= 0 ? totalItems - 1 : selectedIndex - 1;
        // Manual scroll trigger
        if (!isScriptMode)
          setTimeout(() => resultsListRef?.scrollToSelected(), 0);
        break;
      case "Enter":
        e.preventDefault();
        if (isScriptMode && scriptResults[selectedIndex]) {
          handleScriptActivate(scriptResults[selectedIndex]);
        } else if (activeRow?.type === "header") {
          resultsListRef?.toggleHeaderAt(selectedIndex);
        } else if (activeResult) {
          handleActivate(activeResult);
        }
        break;
      case "Tab":
        e.preventDefault();
        if (!isScriptMode && results[selectedIndex]) {
          const res = results[selectedIndex];
          if (res.exec.startsWith("fill:")) {
            handleActivate(res);
            break;
          } else if (res.exec.startsWith("install:")) {
            query = "install " + res.exec.slice(8) + " ";
            handleSearch(query);
            setTimeout(() => {
              document.querySelector("input")?.focus();
            }, 10);
            break;
          }
        }

        if (e.shiftKey) {
          selectedIndex =
            selectedIndex <= 0 ? totalItems - 1 : selectedIndex - 1;
        } else {
          selectedIndex = (selectedIndex + 1) % totalItems;
        }

        // Manual scroll trigger to track Tab navigation
        if (!isScriptMode)
          setTimeout(() => resultsListRef?.scrollToSelected(), 0);
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

<div
  class="vanta-root vanta-glass"
  class:css-blur={blurMode === "fallback"}
  role="application"
>
  {#if view === "settings" && vantaConfig}
    <div
      in:fade={{ duration: 150 }}
      style="height: 100%; width: 100%; position: relative;"
    >
      <SettingsView
        bind:config={vantaConfig}
        {availableThemes}
        onClose={() => (view = "launcher")}
      />
    </div>
  {:else if currentMode === "clipboard"}
    <!-- Clipboard mode: full replace, no launcher behind it -->
    <div
      in:fade={{ duration: 150 }}
      style="height: 100%; width: 100%; display: grid; grid-template-rows: auto 1fr auto;"
    >
      <SearchInput
        bind:this={searchInputRef}
        bind:query
        onSearch={(q) => updateClipboardSuggestions(q)}
        onEscape={handleEscape}
      />
      <ResultsList
        bind:this={resultsListRef}
        {results}
        bind:selectedIndex
        onActivate={handleActivate}
        {preferDefaultOrder}
        on:visiblecount={(event) => (visibleRowCount = event.detail.count)}
      />
    </div>
  {:else}
    <div
      in:fade={{ duration: 150 }}
      style="height: 100%; width: 100%; display: grid; grid-template-rows: auto 1fr auto;"
    >
      <SearchInput
        bind:this={searchInputRef}
        bind:query
        onSearch={handleSearch}
        onEscape={handleEscape}
      />

      {#if isScriptMode}
        <!-- Script results -->
        <div
          class="results-container"
          role="listbox"
          aria-label="Script results"
        >
          {#if activeScriptCaps.length}
            <div class="cap-badges" aria-label="Capabilities required">
              {#each activeScriptCaps as cap}
                <span class="cap-badge">
                  <span class={`cap-svg icon-${capIcon(cap)}`}></span>
                  {cap}
                </span>
              {/each}
              <span class="cap-note">Requires permission</span>
            </div>
          {/if}

          {#if scriptResults.length === 0}
            <div class="empty-state">
              <span class="empty-icon">⚡</span>
              <span>
                Running {activeScriptKeyword}...
                {#if activeScriptCaps.length}
                  <small class="cap-footnote">
                    Capabilities: {activeScriptCaps.join(", ")}
                  </small>
                {/if}
              </span>
            </div>
          {:else}
            {#each scriptResults as item, i}
              <ScriptResultItem
                {item}
                index={i}
                isSelected={i === selectedIndex}
                onSelect={(idx) => (selectedIndex = idx)}
                onActivate={handleScriptActivate}
              />
            {/each}
          {/if}
        </div>
      {:else}
        <!-- Normal app results -->
        <ResultsList
          bind:this={resultsListRef}
          {results}
          bind:selectedIndex
          onActivate={handleActivate}
          {preferDefaultOrder}
          on:visiblecount={(event) => (visibleRowCount = event.detail.count)}
        />
      {/if}

      <StatusBar
        resultCount={isScriptMode ? scriptResults.length : visibleRowCount}
        {searchTime}
      />
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

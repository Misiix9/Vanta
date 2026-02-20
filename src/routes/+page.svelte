<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { onMount } from "svelte";
  import type {
    VantaConfig,
    SearchResult,
    BlurStatus,
    ScriptEntry,
    ScriptItem,
    ScriptOutput,
    ThemeMeta,
  } from "$lib/types";
  import { applyTheme } from "$lib/theme";
  import SearchInput from "$lib/components/SearchInput.svelte";
  import ResultsList from "$lib/components/ResultsList.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  import ScriptResultItem from "$lib/components/ScriptResultItem.svelte";
  import SettingsView from "$lib/components/SettingsView.svelte";

  let query = $state("");
  let vantaConfig: VantaConfig | undefined = $state();
  let results: SearchResult[] = $state([]);
  let view: "launcher" | "settings" = $state("launcher");
  let scriptResults: ScriptItem[] = $state([]);
  let isScriptMode = $state(false);
  let activeScriptKeyword = $state("");
  let selectedIndex = $state(0);
  let searchTime: number | null = $state(null);
  let blurMode: string = $state("fallback");
  let searchInputRef: SearchInput | undefined = $state();
  let resultsListRef: ResultsList | undefined = $state();
  let availableScripts: ScriptEntry[] = $state([]);
  let actionInFlight = $state(false); // guard: don't dismiss during actions
  let isVisible = $state(true);
  let isMouseOver = $state(false);

  let availableThemes: ThemeMeta[] = $state([]);

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
    isScriptMode ? scriptResults.length : results.length,
  );

  // Clipboard State
  let currentMode: "launcher" | "clipboard" = $state("launcher");
  let clipboardHistory: any[] = [];

  onMount(async () => {
    // ... existing onMount code ...
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
    });

    // Listen for blur status
    await listen<BlurStatus>("blur-status", (event) => {
      blurMode = event.payload.mode;
    });

    // Listen for script changes (hot-reload)
    await listen<ScriptEntry[]>("scripts-changed", (event) => {
      availableScripts = event.payload;
      console.log("Scripts updated:", availableScripts.length);
    });

    // Listen for Clipboard Shortcut
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
    });

    // ── Click-away dismiss ──
    const appWindow = getCurrentWebviewWindow();
    await appWindow.onFocusChanged(({ payload: focused }) => {
      // Focus logic placeholder
    });

    // Listen for window shown event → refocus the search input and reload suggestions
    await listen("tauri://focus", () => {
      isVisible = true;
      setTimeout(() => {
        searchInputRef?.focus?.();
        if (query.trim() === "" && currentMode === "launcher") {
          loadSuggestions();
        }
      }, 50);
    });

    loadSuggestions();
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
      }));
    }
    selectedIndex = 0;
  }

  /**
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

  async function loadSuggestions() {
    try {
      const suggestions = await invoke<SearchResult[]>("get_suggestions");
      results = suggestions;
      // If we have suggestions, select the first one
      if (results.length > 0) {
        selectedIndex = 0;
      }
    } catch (e) {
      console.error("Failed to load suggestions:", e);
    }
  }

  async function handleSearch(q: string) {
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
      results = [];

      try {
        const start = performance.now();
        const output = await invoke<ScriptOutput>("execute_script", {
          keyword,
          args,
        });
        const elapsed = performance.now() - start;

        scriptResults = output.items;
        selectedIndex = 0;
        searchTime = elapsed;
      } catch (e) {
        console.error("Script execution failed:", e);
        // Show error as a script result
        scriptResults = [
          {
            title: `Script Error: ${keyword}`,
            subtitle: String(e),
            icon: "error",
            urgency: "critical",
          },
        ];
        searchTime = null;
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
      const elapsed = performance.now() - start;

      results = searchResults;
      selectedIndex = 0;
      searchTime = elapsed;
    } catch (e) {
      console.error("Search failed:", e);
      results = [];
      searchTime = null;
    }
  }

  async function handleActivate(result: SearchResult) {
    actionInFlight = true;
    try {
      if (result.exec.startsWith("fill:")) {
        query = result.exec.slice(5);
        await handleSearch(query);
        setTimeout(() => {
          document.querySelector("input")?.focus();
        }, 10);
        return;
      } else if (result.exec.startsWith("copy:")) {
        const value = result.exec.slice(5);
        await navigator.clipboard.writeText(value);
        resetAndHide();
      } else if (result.exec.startsWith("install:")) {
        await invoke("launch_app", { exec: result.exec });
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
      setTimeout(() => (actionInFlight = false), 200);
    }
  }

  async function handleScriptActivate(item: ScriptItem) {
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
      setTimeout(() => (actionInFlight = false), 200);
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
    if (currentMode === "clipboard" && query === "") {
      // Close the window entirely instead of staying in launcher
      resetAndHide();
      return;
    }
    resetAndHide();
  }

  function handleKeydown(e: KeyboardEvent) {
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

    // Launcher logic
    if (totalItems === 0) return;

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
        } else if (results[selectedIndex]) {
          handleActivate(results[selectedIndex]);
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
        break;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="vanta-root vanta-glass"
  class:css-blur={blurMode === "fallback"}
  role="application"
>
  {#if view === "settings" && vantaConfig}
    <SettingsView
      bind:config={vantaConfig}
      {availableThemes}
      onClose={() => (view = "launcher")}
    />
  {:else if currentMode === "clipboard"}
    <!-- Clipboard mode: full replace, no launcher behind it -->
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
    />
  {:else}
    <SearchInput
      bind:this={searchInputRef}
      bind:query
      onSearch={handleSearch}
      onEscape={handleEscape}
    />

    {#if isScriptMode}
      <!-- Script results -->
      <div class="results-container" role="listbox" aria-label="Script results">
        {#if scriptResults.length === 0}
          <div class="empty-state">
            <span class="empty-icon">⚡</span>
            <span>Running {activeScriptKeyword}...</span>
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
      />
    {/if}

    <StatusBar
      resultCount={isScriptMode ? scriptResults.length : results.length}
      {searchTime}
    />
  {/if}
</div>

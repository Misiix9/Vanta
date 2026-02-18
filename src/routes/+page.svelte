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
  } from "$lib/types";
  import { applyTheme } from "$lib/theme";
  import SearchInput from "$lib/components/SearchInput.svelte";
  import ResultsList from "$lib/components/ResultsList.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  import ScriptResultItem from "$lib/components/ScriptResultItem.svelte";

  let query = $state("");
  let results: SearchResult[] = $state([]);
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

  // Derived: total item count for navigation
  let totalItems = $derived(
    isScriptMode ? scriptResults.length : results.length,
  );

  // Load config on mount and listen for updates
  onMount(async () => {
    try {
      const config = await invoke<VantaConfig>("get_config");
      applyTheme(config);
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
      applyTheme(event.payload);
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

    // ── Click-away dismiss ──
    // Currently disabled to support "Focus Follows Mouse" workflows
    const appWindow = getCurrentWebviewWindow();
    await appWindow.onFocusChanged(({ payload: focused }) => {
      // Focus logic placeholder
    });

    // Listen for window shown event → refocus the search input and reload suggestions
    await listen("tauri://focus", () => {
      isVisible = true;
      // Re-focus search input when window is shown
      setTimeout(() => {
        searchInputRef?.focus?.();
        // Refresh suggestions to reflect recent usage
        if (query.trim() === "") {
          loadSuggestions();
        }
      }, 50);
    });

    // Initial load of suggestions
    loadSuggestions();
  });

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
      await invoke("launch_app", { exec: result.exec });
      resetAndHide();
    } catch (e) {
      console.error("Launch failed:", e);
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
    activeScriptKeyword = "";
    selectedIndex = 0;
    searchTime = null;
    isVisible = false;
    invoke("hide_window").catch((e) => console.error("Hide failed:", e));
  }

  async function handleEscape() {
    resetAndHide();
  }

  function handleKeydown(e: KeyboardEvent) {
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
  class="vanta-shell vanta-glass"
  class:css-blur={blurMode === "fallback"}
  onmouseenter={() => (isMouseOver = true)}
  onmouseleave={() => (isMouseOver = false)}
  role="application"
>
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
</div>

<style>
  .vanta-shell {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .vanta-shell.css-blur {
    backdrop-filter: blur(var(--vanta-blur));
    -webkit-backdrop-filter: blur(var(--vanta-blur));
  }

  .results-container {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 4px 8px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 40px 20px;
    color: var(--vanta-text-dim);
    font-size: 14px;
  }

  .empty-icon {
    font-size: 28px;
    opacity: 0.5;
  }
</style>

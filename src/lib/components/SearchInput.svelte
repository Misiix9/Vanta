<script lang="ts">

  let {
    query = $bindable(""),
    placeholder = "Search apps, files, scripts…",
    queryHistory = [],
    onSearch,
    onEscape,
    onHistoryRecall,
  }: {
    query: string;
    placeholder?: string;
    queryHistory?: string[];
    onSearch: (query: string) => void;
    onEscape: () => void;
    onHistoryRecall?: (query: string) => void;
  } = $props();

  let inputEl: HTMLInputElement | undefined = $state();
  let debounceTimer: ReturnType<typeof setTimeout> | undefined;
  let historyIndex = $state(-1);

  function handleInput() {
    historyIndex = -1;
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      onSearch(query);
    }, 50);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onEscape();
      return;
    }
    // Up-arrow in empty input or at beginning recalls query history.
    if (e.key === "ArrowUp" && query.trim() === "" && queryHistory.length > 0) {
      e.preventDefault();
      const next = Math.min(historyIndex + 1, queryHistory.length - 1);
      if (next !== historyIndex) {
        historyIndex = next;
        query = queryHistory[historyIndex];
        onHistoryRecall?.(query);
      }
      return;
    }
    if (e.key === "ArrowDown" && historyIndex >= 0) {
      e.preventDefault();
      historyIndex -= 1;
      if (historyIndex < 0) {
        query = "";
        onSearch("");
      } else {
        query = queryHistory[historyIndex];
        onHistoryRecall?.(query);
      }
      return;
    }
  }

  export function focus() {
    inputEl?.focus();
  }

  $effect(() => {
    // Auto-focus on mount
    inputEl?.focus();
  });
</script>

<div class="search-bar">
  <div class="vanta-input-wrap">
    <input
      bind:this={inputEl}
      bind:value={query}
      oninput={handleInput}
      onkeydown={handleKeydown}
      class="vanta-input"
      class:empty={query.length === 0}
      type="text"
      placeholder={placeholder}
      spellcheck="false"
      autocomplete="off"
      aria-label="Search applications, files, and commands"
    />
    {#if query.length === 0}
      <span class="vanta-caret" aria-hidden="true"></span>
    {/if}
  </div>

  {#if query.length > 0}
    <button
      class="clear-btn"
      aria-label="Clear search"
      onclick={() => {
        query = "";
        onSearch("");
        inputEl?.focus();
      }}
    >
      <svg
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="M18 6 6 18" />
        <path d="m6 6 12 12" />
      </svg>
    </button>
  {/if}
</div>



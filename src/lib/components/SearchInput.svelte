<script lang="ts">
  import type { SearchResult } from "$lib/types";
  import { createEventDispatcher } from "svelte";

  let {
    query = $bindable(""),
    onSearch,
    onEscape,
  }: {
    query: string;
    onSearch: (query: string) => void;
    onEscape: () => void;
  } = $props();

  let inputEl: HTMLInputElement | undefined = $state();
  let debounceTimer: ReturnType<typeof setTimeout> | undefined;

  function handleInput() {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      onSearch(query);
    }, 50);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onEscape();
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
  <!-- No search icon, purely text based for minimalism -->
  <input
    bind:this={inputEl}
    bind:value={query}
    oninput={handleInput}
    onkeydown={handleKeydown}
    class="vanta-input"
    type="text"
    placeholder=" "
    spellcheck="false"
    autocomplete="off"
    autofocus
  />

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



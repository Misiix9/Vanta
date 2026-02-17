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

<div class="search-wrapper">
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

<style>
  .search-wrapper {
    width: 100%;
    padding: 24px 24px 12px 24px;
    position: relative;
    display: flex;
    align-items: center;
  }

  .vanta-input {
    width: 100%;
    background: transparent;
    border: none;
    outline: none;

    /* Typography */
    font-family: "SN Pro", "Inter", sans-serif;
    font-size: 24px;
    font-weight: 700;
    color: var(--vanta-text);

    /* Custom Caret */
    caret-color: var(--vanta-accent);
  }

  .vanta-input::placeholder {
    color: var(--vanta-text-dim);
    opacity: 0.3;
  }

  /* Clear button - Keep it, but minimal */
  .clear-btn {
    background: none;
    border: none;
    color: var(--vanta-text-dim);
    cursor: pointer;
    padding: 8px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    transition: opacity 0.2s ease;
  }

  .search-wrapper:hover .clear-btn,
  .vanta-input:focus + .clear-btn {
    opacity: 0.5;
  }

  .clear-btn:hover {
    opacity: 1 !important;
    background: rgba(255, 255, 255, 0.1);
  }
</style>

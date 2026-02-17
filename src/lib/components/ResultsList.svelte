<script lang="ts">
  import type { SearchResult } from "$lib/types";
  import ResultItem from "./ResultItem.svelte";

  let {
    results = [],
    selectedIndex = $bindable(0),
    onActivate,
  }: {
    results: SearchResult[];
    selectedIndex: number;
    onActivate: (result: SearchResult) => void;
  } = $props();

  function handleSelect(index: number) {
    selectedIndex = index;
  }
</script>

{#if results.length > 0}
  <div class="results-container" role="listbox">
    {#each results as result, i (result.exec + result.title)}
      <ResultItem
        {result}
        index={i}
        isSelected={i === selectedIndex}
        onSelect={handleSelect}
        {onActivate}
      />
    {/each}
  </div>
{:else}
  <div class="empty-state">
    <div class="empty-icon">
      <svg
        width="32"
        height="32"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
        opacity="0.3"
      >
        <circle cx="11" cy="11" r="8" />
        <path d="m21 21-4.3-4.3" />
      </svg>
    </div>
    <p class="empty-text">Type to search applications...</p>
  </div>
{/if}

<style>
  .results-container {
    padding: 6px;
    overflow-y: auto;
    overflow-x: hidden;
    max-height: calc(420px - 70px - 36px);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px 20px;
    gap: 12px;
  }

  .empty-icon {
    color: var(--vanta-text-dim);
  }

  .empty-text {
    color: var(--vanta-text-dim);
    font-size: 13px;
    opacity: 0.5;
  }
</style>

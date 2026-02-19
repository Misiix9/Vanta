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

  let container: HTMLDivElement | null = $state(null);
  let itemElements: (HTMLDivElement | null)[] = $state([]);

  let scrollTop = $state(0);
  let scrollHeight = $state(0);
  let clientHeight = $state(0);
  let isHoveringScrollbar = $state(false);
  let isScrolling = $state(false);
  let scrollTimeout: any = null;

  function handleScroll() {
    if (container) {
      scrollTop = container.scrollTop;
      scrollHeight = container.scrollHeight;
      clientHeight = container.clientHeight;

      // Hide scrollbar after inactivity
      isScrolling = true;
      if (scrollTimeout) clearTimeout(scrollTimeout);
      scrollTimeout = setTimeout(() => {
        isScrolling = false;
      }, 1000);
    }
  }

  // Update metrics on resize or content change
  $effect(() => {
    if (container) {
      // Initial read
      handleScroll();
      // Observer for resize
      const ro = new ResizeObserver(handleScroll);
      ro.observe(container);
      return () => ro.disconnect();
    }
  });

  // Calculate thumb metrics
  let thumbHeight = $derived.by(() => {
    if (!clientHeight || !scrollHeight) return 0;
    const ratio = clientHeight / scrollHeight;
    return Math.max(40, clientHeight * ratio);
  });

  let thumbTop = $derived.by(() => {
    if (!clientHeight || !scrollHeight || thumbHeight === 0) return 0;
    const maxScroll = scrollHeight - clientHeight;
    const scrollRatio = scrollTop / maxScroll;
    return scrollRatio * (clientHeight - thumbHeight);
  });

  let showScrollbar = $derived(scrollHeight > clientHeight);

  function handleSelect(index: number) {
    selectedIndex = index;
  }

  export function scrollToSelected() {
    if (selectedIndex >= 0 && results.length > 0 && container) {
      const el = itemElements[selectedIndex];
      if (el) {
        const itemTop = el.offsetTop;
        const itemBottom = itemTop + el.offsetHeight;
        const containerHeight = container.clientHeight;
        const containerScrollTop = container.scrollTop;

        // Scroll to item if out of view (smoothly)
        if (itemTop < containerScrollTop) {
          container.scrollTo({ top: itemTop, behavior: "smooth" });
        } else if (itemBottom > containerScrollTop + containerHeight) {
          container.scrollTo({
            top: itemBottom - containerHeight,
            behavior: "smooth",
          });
        }
      }
    }
  }
</script>

{#if results.length > 0}
  <div class="results-wrapper">
    <div
      class="results-container no-scrollbar"
      bind:this={container}
      onscroll={handleScroll}
      role="listbox"
    >
      {#each results as result, i (result.id || result.exec + result.title)}
        <div
          bind:this={itemElements[i]}
          id="result-item-{i}"
          style="width: 100%"
        >
          <ResultItem
            {result}
            index={i}
            isSelected={i === selectedIndex}
            onSelect={handleSelect}
            {onActivate}
          />
        </div>
      {/each}
    </div>

    {#if showScrollbar}
      <div
        class="custom-scrollbar-track"
        role="presentation"
        onmouseenter={() => (isHoveringScrollbar = true)}
        onmouseleave={() => (isHoveringScrollbar = false)}
      >
        <div
          class="custom-scrollbar-thumb"
          style:height="{thumbHeight}px"
          style:transform="translateY({thumbTop}px)"
          style:opacity={isHoveringScrollbar || isScrolling ? 1 : 0}
        ></div>
      </div>
    {/if}
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
  .results-wrapper {
    position: relative;
    width: 100%;
    /* Ensure height matches container max-height logic roughly so wrapper contains flex */
    display: flex;
    flex-direction: column;
    overflow: hidden; /* Hide anything leaking */
  }

  .results-container {
    padding: 6px;
    overflow-y: auto;
    overflow-x: hidden;
    max-height: calc(420px - 70px - 36px);
    display: flex;
    flex-direction: column;
    gap: 2px;
    position: relative;
    scroll-behavior: smooth;

    /* Force hide implementation detail of scrollbar */
    scrollbar-width: none !important;
    -ms-overflow-style: none !important;
  }

  .results-container::-webkit-scrollbar {
    display: none !important;
    width: 0 !important;
    height: 0 !important;
    background: transparent !important;
  }

  /* Custom Scrollbar Styles */
  .custom-scrollbar-track {
    position: absolute;
    top: 6px; /* Match padding */
    bottom: 6px;
    right: 4px; /* Padding from edge */
    width: 10px; /* Thicker track area */
    pointer-events: none;
    z-index: 10;
  }

  .custom-scrollbar-thumb {
    width: 100%;
    /* Dark semi-transparent thumb for stealth look */
    background: rgba(0, 0, 0, 0.6);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 9999px;
    transition:
      opacity 0.3s ease,
      transform 0.1s linear,
      background-color 0.2s;
  }

  .custom-scrollbar-thumb:hover {
    background: rgba(0, 0, 0, 0.8);
    border-color: rgba(255, 255, 255, 0.2);
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

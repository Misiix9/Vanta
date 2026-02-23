<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { SearchResult } from "$lib/types";
  import ResultItem from "./ResultItem.svelte";
  type VisibleRow =
    | {
      type: "header";
      label: string;
      count: number;
      key: string;
      collapsed: boolean;
      }
    | {
        type: "item";
        result: SearchResult;
        key: string;
        groupLabel: string;
        itemIndex: number;
      };

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
  const dispatch = createEventDispatcher<{ visiblecount: { count: number } }>();
  let collapsedGroups = $state(new Set<string>());

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

  let groupedResults = $derived.by(() => {
    const order: { label: string; items: SearchResult[] }[] = [];
    const seen = new Map<string, { label: string; items: SearchResult[] }>();

    for (const res of results) {
      const label = res.group?.trim() || "Ungrouped";
      let group = seen.get(label);
      if (!group) {
        group = { label, items: [] };
        seen.set(label, group);
        order.push(group);
      }
      group.items.push(res);
    }

    return order;
  });

  let visibleRows = $derived.by<VisibleRow[]>(() => {
    const rows: VisibleRow[] = [];
    let flatItemIndex = 0;

    groupedResults.forEach((group, groupIndex) => {
      const isCollapsed = collapsedGroups.has(group.label);
      rows.push({
        type: "header",
        label: group.label,
        count: group.items.length,
        key: `header-${groupIndex}-${group.label}`,
        collapsed: isCollapsed,
      });

      if (!isCollapsed) {
        group.items.forEach((result, itemIndex) => {
          const keyBase = result.id ?? `${result.exec}-${groupIndex}-${itemIndex}`;
          rows.push({
            type: "item",
            result,
            groupLabel: group.label,
            itemIndex: flatItemIndex++,
            key: `item-${groupIndex}-${itemIndex}-${keyBase}`,
          });
        });
      }
    });

    return rows;
  });

  $effect(() => {
    itemElements.length = visibleRows.length;
    dispatch("visiblecount", { count: visibleRows.length });
  });

  function handleSelect(index: number) {
    if (index < 0 || index >= visibleRows.length) return;
    selectedIndex = index;
  }

  export function toggleHeaderAt(index: number) {
    const row = visibleRows[index];
    if (!row || row.type !== "header") return;

    const next = new Set(collapsedGroups);
    if (next.has(row.label)) {
      next.delete(row.label);
    } else {
      next.add(row.label);
    }
    collapsedGroups = next;
  }

  export function getVisibleRow(index: number) {
    return visibleRows[index];
  }

  export function scrollToSelected() {
    if (
      selectedIndex >= 0 &&
      selectedIndex < visibleRows.length &&
      container
    ) {
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

{#if visibleRows.length > 0}
  <div class="results-wrapper">
    <div
      class="results-container no-scrollbar"
      bind:this={container}
      onscroll={handleScroll}
      role="listbox"
    >
      {#each visibleRows as row, i (row.key)}
        {#if row.type === "header"}
          <div
            class="group-header"
            class:selected={i === selectedIndex}
            bind:this={itemElements[i]}
            id="result-row-{i}"
            role="button"
            tabindex="0"
            aria-expanded={!row.collapsed}
            onmouseenter={() => handleSelect(i)}
            onclick={() => toggleHeaderAt(i)}
            onkeydown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                toggleHeaderAt(i);
              }
            }}
          >
            <span class="group-title">
              <span class={`chevron ${row.collapsed ? "collapsed" : ""}`}>▸</span>
              {row.label}
            </span>
            <span class="group-count">{row.count}</span>
          </div>
        {:else}
          <div
            bind:this={itemElements[i]}
            id="result-row-{i}"
            style="width: 100%"
          >
            <ResultItem
              result={row.result}
              index={i}
              displayIndex={row.itemIndex}
              isSelected={i === selectedIndex}
              onSelect={handleSelect}
              {onActivate}
            />
          </div>
        {/if}
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
  .group-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    font-size: 12px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: rgba(255, 255, 255, 0.7);
    background: rgba(255, 255, 255, 0.02);
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
    transition: background 140ms ease, color 140ms ease;
  }

  .group-header.selected {
    background: rgba(255, 255, 255, 0.08);
    color: rgba(255, 255, 255, 0.95);
  }

  .group-title {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
  }

  .chevron {
    display: inline-block;
    transition: transform 140ms ease;
  }

  .chevron.collapsed {
    transform: rotate(90deg);
  }

  .group-count {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 2px 8px;
    border-radius: 999px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    font-size: 11px;
    color: rgba(255, 255, 255, 0.8);
    background: rgba(255, 255, 255, 0.04);
  }
</style>



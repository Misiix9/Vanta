<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { SearchResult, ResultAction } from "$lib/types";
  import ResultItem from "./ResultItem.svelte";
  import SectionHeader from "./SectionHeader.svelte";
  type VisibleRow =
    | {
      type: "header";
      label: string;
      count: number;
      key: string;
      }
    | {
        type: "subheader";
        label: string;
        key: string;
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
    groupBySection = true,
    query = "",
    showScore = false,
    onActivate,
    onActionClick,
    onContextMenu,
  }: {
    results: SearchResult[];
    selectedIndex: number;
    groupBySection?: boolean;
    query?: string;
    showScore?: boolean;
    onActivate: (result: SearchResult) => void;
    onActionClick?: (result: SearchResult, action: ResultAction) => void;
    onContextMenu?: (e: MouseEvent, result: SearchResult) => void;
  } = $props();

  let container: HTMLDivElement | null = $state(null);
  let itemElements: (HTMLDivElement | null)[] = $state([]);
  const dispatch = createEventDispatcher<{ visiblecount: { count: number } }>();

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

  const sectionOrder = [
    "Running",
    "Apps",
    "Documents",
    "Macros",
    "Scripts",
    "Calculator",
    "Clipboard",
    "Settings",
    "Other",
    "Commands",
  ];

  function deriveSection(res: SearchResult): string {
    if (res.section && res.section.trim()) return res.section.trim();

    if (typeof res.source === "object" && "Script" in res.source) {
      return "Scripts";
    }

    switch (res.source) {
      case "Application":
        return "Apps";
      case "Window":
        return "Running";
      case "File":
        return "Documents";
      case "Calculator":
        return "Calculator";
      case "Clipboard":
        return "Clipboard";
      default:
        return "Other";
    }
  }

  let groupedResults = $derived.by(() => {
    const seen = new Map<string, { label: string; items: SearchResult[] }>();

    for (const res of results) {
      const label = deriveSection(res);
      const group = seen.get(label) ?? { label, items: [] };
      group.items.push(res);
      seen.set(label, group);
    }

    // Only show sections that currently have results, while preserving known order.
    const ordered: { label: string; items: SearchResult[] }[] = [];
    for (const label of sectionOrder) {
      const group = seen.get(label);
      if (group && group.items.length > 0) ordered.push(group);
    }

    const extras = Array.from(seen.values()).filter(
      (g) => !sectionOrder.includes(g.label) && g.items.length > 0,
    );
    extras.sort((a, b) => a.label.localeCompare(b.label));

    return [...ordered, ...extras];
  });

  let visibleRows = $derived.by<VisibleRow[]>(() => {
    const rows: VisibleRow[] = [];

    if (!groupBySection) {
      results.forEach((result, idx) => {
        const keyBase = result.id ?? `${result.exec}-${idx}`;
        rows.push({
          type: "item",
          result,
          groupLabel: "Results",
          itemIndex: idx,
          key: `item-flat-${idx}-${keyBase}`,
        });
      });
      return rows;
    }

    let flatItemIndex = 0;

    groupedResults.forEach((group, groupIndex) => {
      rows.push({
        type: "header",
        label: group.label,
        count: group.items.length,
        key: `header-${groupIndex}-${group.label}`,
      });

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
    });

    return rows;
  });

  // Keep itemElements aligned with visible rows so bind:element never receives undefined.
  $effect(() => {
    if (itemElements.length !== visibleRows.length) {
      itemElements = Array(visibleRows.length).fill(null);
    }

    if (visibleRows.length === 0) {
      selectedIndex = 0;
    } else if (selectedIndex >= visibleRows.length) {
      selectedIndex = visibleRows.length - 1;
    } else if (selectedIndex < 0) {
      selectedIndex = 0;
    }

    dispatch("visiblecount", { count: visibleRows.length });
  });

  // Keep the selected row anchored in view as selection changes via keys or hover.
  $effect(() => {
    scrollToSelected();
  });

  function handleSelect(index: number) {
    if (index < 0 || index >= visibleRows.length) return;
    selectedIndex = index;
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
  <div class="results-wrapper" class:grouped={groupBySection} class:flat={!groupBySection} id="vanta-results">
    <div
      class="results-container no-scrollbar"
      bind:this={container}
      onscroll={handleScroll}
      role="listbox"
    >
      {#each visibleRows as row, i (row.key)}
        {#if row.type === "header"}
          <SectionHeader
            bind:element={itemElements[i]}
            label={row.label}
            count={row.count}
            isSelected={i === selectedIndex}
          />
        {:else if row.type === "subheader"}
          <div
            class="subheader"
            bind:this={itemElements[i]}
            aria-hidden="true"
          >
            {row.label}
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
              {showScore}
              onSelect={handleSelect}
              {onActivate}
              {onActionClick}
              {onContextMenu}
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
    {#if query.trim()}
      {#if query.startsWith("type:")}
        <p class="empty-text">No results for this source filter. Try a different filter or broader query.</p>
      {:else}
        <p class="empty-text">No results for "{query}". Try a different spelling or use <kbd>type:app</kbd> to filter.</p>
      {/if}
    {:else}
      <p class="empty-text">Start typing to search apps, files, and more.</p>
      <div class="empty-hints">
        <span class="empty-hint-chip">type:app</span>
        <span class="empty-hint-chip">type:file</span>
        <span class="empty-hint-chip">in:clipboard</span>
      </div>
    {/if}
  </div>
{/if}



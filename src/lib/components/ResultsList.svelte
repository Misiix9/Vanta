<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { SearchResult } from "$lib/types";
  import ResultItem from "./ResultItem.svelte";
  import SectionHeader from "./SectionHeader.svelte";
  type VisibleRow =
    | {
      type: "header";
      label: string;
      count: number;
      key: string;
      collapsed: boolean;
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
    onActivate,
    preferDefaultOrder = false,
  }: {
    results: SearchResult[];
    selectedIndex: number;
    onActivate: (result: SearchResult) => void;
    preferDefaultOrder?: boolean;
  } = $props();

  let container: HTMLDivElement | null = $state(null);
  let itemElements: (HTMLDivElement | null)[] = $state([]);
  const dispatch = createEventDispatcher<{ visiblecount: { count: number } }>();
  let collapsedGroups = $state(new Set<string>());
  // Remember if the user explicitly toggled Running so we don't auto-collapse it again.
  let runningOverride: boolean | null = $state(null);

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
    const order: { label: string; items: SearchResult[] }[] = [];
    const seen = new Map<string, { label: string; items: SearchResult[] }>();

    for (const res of results) {
      const label = deriveSection(res);
      let group = seen.get(label);
      if (!group) {
        group = { label, items: [] };
        seen.set(label, group);
        order.push(group);
      }
      group.items.push(res);
    }

    // Always surface a Running group so the header is visible even if no window rows are present.
    if (!seen.has("Running")) {
      const placeholder = { label: "Running", items: [] };
      seen.set("Running", placeholder);
      order.push(placeholder);
    }

    order.sort((a, b) => {
      const ai = sectionOrder.indexOf(a.label);
      const bi = sectionOrder.indexOf(b.label);
      const aIdx = ai === -1 ? sectionOrder.length : ai;
      const bIdx = bi === -1 ? sectionOrder.length : bi;

      // Always enforce the explicit section priority so Apps and Documents stay on top.
      if (aIdx !== bIdx) return aIdx - bIdx;

      // Within the same priority bucket, fall back to the highest scored item for stability.
      const aTop = a.items.reduce((max, item) => Math.max(max, item.score ?? 0), -Infinity);
      const bTop = b.items.reduce((max, item) => Math.max(max, item.score ?? 0), -Infinity);
      if (aTop !== bTop) return bTop - aTop;

      return a.label.localeCompare(b.label);
    });

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
        if (group.label === "Running") {
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
        } else {
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
      }
    });

    return rows;
  });

  // Keep itemElements aligned with visible rows so bind:element never receives undefined.
  $effect(() => {
    if (itemElements.length !== visibleRows.length) {
      itemElements = Array(visibleRows.length).fill(null);
    }
    dispatch("visiblecount", { count: visibleRows.length });
  });

  // Keep the selected row anchored in view as selection changes via keys or hover.
  $effect(() => {
    scrollToSelected();
  });

  // Avoid selecting section headers; jump to the first real item when results change.
  $effect(() => {
    if (!visibleRows.length) return;
    const row = visibleRows[selectedIndex];
    if (row?.type === "header") {
      const firstItemIndex = visibleRows.findIndex((r) => r.type === "item");
      if (firstItemIndex >= 0) selectedIndex = firstItemIndex;
    }
  });

  // Default-collapse Running when showing idle suggestions, expand when actively searching.
  // If the user toggles it, respect their override until they toggle again.
  $effect(() => {
    const next = new Set(collapsedGroups);
    const desiredCollapsed =
      runningOverride !== null ? runningOverride : preferDefaultOrder;

    if (desiredCollapsed && !next.has("Running")) {
      next.add("Running");
      collapsedGroups = next;
    }

    if (!desiredCollapsed && next.has("Running")) {
      next.delete("Running");
      collapsedGroups = next;
    }
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
      if (row.label === "Running") runningOverride = false;
    } else {
      next.add(row.label);
      if (row.label === "Running") runningOverride = true;
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
          <SectionHeader
            bind:element={itemElements[i]}
            label={row.label}
            count={row.count}
            collapsed={row.collapsed}
            selected={i === selectedIndex}
            on:hover={() => handleSelect(i)}
            on:toggle={() => toggleHeaderAt(i)}
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



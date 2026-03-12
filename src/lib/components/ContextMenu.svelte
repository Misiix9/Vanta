<script lang="ts">
  import type { SearchResult, ResultAction } from "$lib/types";
  import { execForAction } from "$lib/ranking";

  let {
    result,
    x,
    y,
    onAction,
    onClose,
  }: {
    result: SearchResult;
    x: number;
    y: number;
    onAction: (result: SearchResult, exec: string, command?: ResultAction["command"]) => void;
    onClose: () => void;
  } = $props();

  interface MenuItem {
    label: string;
    exec: string;
    command?: ResultAction["command"];
    divider?: boolean;
  }

  let menuItems = $derived.by<MenuItem[]>(() => {
    const items: MenuItem[] = [];

    // Primary action
    const primaryLabel =
      result.source === "Calculator" ? "Copy Result"
      : result.source === "Window" ? "Focus Window"
      : result.source === "Clipboard" ? "Copy"
      : result.source === "File" ? "Open"
      : "Launch";

    items.push({ label: primaryLabel, exec: result.exec, command: result.command });

    // Existing actions from result
    if (result.actions?.length) {
      for (const action of result.actions) {
        items.push({
          label: action.label,
          exec: execForAction(result, action),
          command: action.command,
          divider: items.length === 1,
        });
      }
    }

    return items;
  });

  function handleClick(item: MenuItem) {
    onAction(result, item.exec, item.command);
    onClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }

  // Adjust position so the menu stays within the window
  let adjustedX = $derived(Math.min(x, window.innerWidth - 200));
  let adjustedY = $derived(Math.min(y, window.innerHeight - (menuItems.length * 32 + 16)));
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="context-menu-backdrop" onclick={onClose} onkeydown={handleKeydown}>
  <div
    class="context-menu"
    role="menu"
    tabindex="-1"
    style="left: {adjustedX}px; top: {adjustedY}px;"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >
    {#each menuItems as item, i}
      {#if item.divider}
        <div class="context-menu-divider" role="separator"></div>
      {/if}
      <button
        class="context-menu-item"
        role="menuitem"
        onclick={() => handleClick(item)}
      >
        {item.label}
      </button>
    {/each}
  </div>
</div>

<script lang="ts">
  import { onMount } from "svelte";

  let { onClose }: { onClose: () => void } = $props();

  let modalEl: HTMLDivElement | null = null;
  let closeBtn: HTMLButtonElement | null = null;

  const shortcuts: Array<{ label: string; keys: string }> = [
    { label: "Open / focus Vanta", keys: "Super" },
    { label: "Navigate results", keys: "↑ / ↓" },
    { label: "Activate result", keys: "Enter" },
    { label: "Secondary action", keys: "Shift + Enter" },
    { label: "Tertiary action", keys: "Alt + Enter" },
    { label: "Copy path", keys: "Ctrl + Shift + C" },
    { label: "Cycle results", keys: "Tab / Shift + Tab" },
    { label: "Open settings", keys: "Ctrl + ," },
    { label: "Show shortcuts", keys: "Ctrl + ?" },
    { label: "Dismiss / go back", keys: "Escape" },
    { label: "Skip to results", keys: "Tab (from search)" },
  ];

  onMount(() => {
    closeBtn?.focus();
  });

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      onClose();
      return;
    }
    if (event.key === "Tab" && modalEl) {
      const focusables = Array.from(
        modalEl.querySelectorAll<HTMLElement>("button:not(:disabled)")
      ).filter((el) => el.offsetParent !== null);
      if (focusables.length === 0) return;
      const currentIndex = focusables.indexOf(document.activeElement as HTMLElement);
      const nextIndex = event.shiftKey
        ? (currentIndex <= 0 ? focusables.length - 1 : currentIndex - 1)
        : (currentIndex + 1) % focusables.length;
      focusables[nextIndex]?.focus();
      event.preventDefault();
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="shortcuts-overlay" onkeydown={handleKeydown}>
  <div
    class="shortcuts-modal"
    bind:this={modalEl}
    role="dialog"
    aria-modal="true"
    aria-label="Keyboard shortcuts"
  >
    <h3>Keyboard Shortcuts</h3>
    {#each shortcuts as { label, keys }}
      <div class="shortcut-row">
        <span>{label}</span>
        <kbd>{keys}</kbd>
      </div>
    {/each}
    <div class="shortcuts-footer">
      <button bind:this={closeBtn} onclick={onClose}>Close</button>
    </div>
  </div>
</div>

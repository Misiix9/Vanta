<script lang="ts">
  import { onMount } from "svelte";
  import type { Capability } from "$lib/types";

  let {
    scriptId,
    missingCaps = [],
    onAllowOnce,
    onAllowAlways,
    onDeny,
    busy = false,
  }: {
    scriptId: string;
    missingCaps: Capability[];
    onAllowOnce: () => void;
    onAllowAlways: () => void;
    onDeny: () => void;
    busy?: boolean;
  } = $props();

  let modalEl: HTMLDivElement | null = null;
  let primaryBtn: HTMLButtonElement | null = null;

  onMount(() => {
    primaryBtn?.focus();
  });

  const capCopy: Record<Capability, { title: string; desc: string; icon: string }> = {
    Network: {
      title: "Network",
      desc: "Connects to the internet or local network endpoints.",
      icon: "network",
    },
    Shell: {
      title: "Shell",
      desc: "Spawns shell commands or child processes.",
      icon: "shell",
    },
    Filesystem: {
      title: "Filesystem",
      desc: "Reads or writes files on your machine.",
      icon: "filesystem",
    },
  };

  function handleKeydown(event: KeyboardEvent) {
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

    // Hard-block: do not close on Escape
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
    }
  }
</script>

<div
  class="permission-overlay"
  role="presentation"
  tabindex="-1"
  onkeydown={handleKeydown}
>
  <div class="permission-modal" bind:this={modalEl} role="dialog" aria-modal="true" aria-label="Extension permission request">
    <div class="header">
      <div class="pill">Permission Needed</div>
      <div class="script-id">{scriptId}</div>
    </div>

    <p class="lede">This extension wants elevated capabilities. Choose how you want to proceed.</p>

    <div class="cap-grid">
      {#each missingCaps as cap}
        <div class="cap-card">
          <div class="cap-icon">
            <span class={`cap-svg icon-${capCopy[cap].icon}`}></span>
          </div>
          <div class="cap-body">
            <div class="cap-title">{capCopy[cap].title}</div>
            <div class="cap-desc">{capCopy[cap].desc}</div>
          </div>
        </div>
      {/each}
    </div>

    <div class="actions">
      <button
        class="btn primary"
        onclick={onAllowOnce}
        bind:this={primaryBtn}
        disabled={busy}
      >
        Allow Once
      </button>
      <button class="btn ghost" onclick={onAllowAlways} disabled={busy}>Always Allow</button>
      <button class="btn danger" onclick={onDeny} disabled={busy}>Deny</button>
    </div>

    <p class="footnote">Hard block: you must choose. Close/escape are disabled until you decide.</p>
  </div>
</div>


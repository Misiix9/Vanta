<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import type { ClipboardItem } from "$lib/types";

  let {
    onEscape,
  }: {
    onEscape: () => void;
  } = $props();

  let items: ClipboardItem[] = $state([]);
  let query = $state("");
  let selectedId: number | null = $state(null);
  let listEl: HTMLDivElement | null = $state(null);
  let dedupeEnabled = $state(true);
  let groupByType = $state(true);
  let maskSensitive = $state(true);
  let revealSensitiveId: number | null = $state(null);

  const DEDUPE_KEY = "vanta.clipboard.dedupe";
  const GROUP_KEY = "vanta.clipboard.groupByType";
  const MASK_KEY = "vanta.clipboard.maskSensitive";

  onMount(() => {
    dedupeEnabled = localStorage.getItem(DEDUPE_KEY) !== "0";
    groupByType = localStorage.getItem(GROUP_KEY) !== "0";
    maskSensitive = localStorage.getItem(MASK_KEY) !== "0";
  });

  function persistToggles() {
    localStorage.setItem(DEDUPE_KEY, dedupeEnabled ? "1" : "0");
    localStorage.setItem(GROUP_KEY, groupByType ? "1" : "0");
    localStorage.setItem(MASK_KEY, maskSensitive ? "1" : "0");
  }

  async function loadItems() {
    try {
      items = await invoke<ClipboardItem[]>("get_clipboard_history");
      if (items.length > 0 && selectedId === null) {
        selectedId = items[0].id;
      }
    } catch (e) {
      console.error("Failed to load clipboard:", e);
    }
  }

  $effect(() => {
    loadItems();
  });

  let filtered = $derived.by(() => {
    if (!query.trim()) return items;
    const lower = query.toLowerCase();
    return items.filter((i) => i.content.toLowerCase().includes(lower));
  });

  let prepared = $derived.by(() => {
    let base = filtered;

    if (dedupeEnabled) {
      const seen = new Set<string>();
      const out: ClipboardItem[] = [];
      for (const item of base) {
        const key = `${item.content_type}::${item.content.trim()}`;
        if (seen.has(key)) continue;
        seen.add(key);
        out.push(item);
      }
      base = out;
    }

    return base;
  });

  let grouped = $derived.by(() => {
    const map = new Map<string, ClipboardItem[]>();

    for (const item of prepared) {
      const typeLabel = badgeFor(item.content_type).label;
      const key = groupByType
        ? `${item.pinned ? "Pinned" : "Recent"} · ${typeLabel}`
        : (item.pinned ? "Pinned" : "Recent");
      const arr = map.get(key) ?? [];
      arr.push(item);
      map.set(key, arr);
    }

    const order = Array.from(map.keys()).sort((a, b) => {
      const aPinned = a.startsWith("Pinned") ? 0 : 1;
      const bPinned = b.startsWith("Pinned") ? 0 : 1;
      if (aPinned !== bPinned) return aPinned - bPinned;
      return a.localeCompare(b);
    });

    return order.map((label) => ({ label, items: map.get(label) ?? [] }));
  });

  let visibleItems = $derived.by(() => grouped.flatMap((g) => g.items));

  let selected = $derived(visibleItems.find((i) => i.id === selectedId) ?? visibleItems[0] ?? null);

  $effect(() => {
    if (selected) {
      selectedId = selected.id;
    }
  });

  $effect(() => {
    if (selectedId !== revealSensitiveId) {
      revealSensitiveId = null;
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onEscape();
      return;
    }

    const idx = visibleItems.findIndex((i) => i.id === selectedId);

    if (e.key === "ArrowDown") {
      e.preventDefault();
      const next = Math.min(idx + 1, visibleItems.length - 1);
      selectedId = visibleItems[next]?.id ?? null;
      scrollToItem(next);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      const prev = Math.max(idx - 1, 0);
      selectedId = visibleItems[prev]?.id ?? null;
      scrollToItem(prev);
    } else if (e.key === "Enter" && selected) {
      e.preventDefault();
      copyToClipboard(selected.content);
    } else if (e.key === "Delete" && selected) {
      e.preventDefault();
      deleteItem(selected.id);
    } else if (e.key.toLowerCase() === "p" && selected) {
      e.preventDefault();
      togglePin(selected.id);
    } else if (e.key.toLowerCase() === "c" && selected) {
      e.preventDefault();
      copyToClipboard(selected.content);
    } else if (e.key.toLowerCase() === "r" && selected) {
      e.preventDefault();
      revealSensitiveId = selected.id;
    }
  }

  function scrollToItem(idx: number) {
    if (!listEl) return;
    const el = listEl.children[idx] as HTMLElement | undefined;
    el?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }

  async function copyToClipboard(content: string) {
    try {
      await navigator.clipboard.writeText(content);
    } catch {
      try {
        await invoke("launch_app", { exec: `copy:${content}` });
      } catch (e) {
        console.error("Copy failed:", e);
      }
    }
  }

  async function deleteItem(id: number) {
    try {
      await invoke("delete_clipboard_item", { id });
      items = items.filter((i) => i.id !== id);
      if (selectedId === id) {
        selectedId = visibleItems[0]?.id ?? null;
      }
    } catch (e) {
      console.error("Delete failed:", e);
    }
  }

  async function togglePin(id: number) {
    try {
      const pinned: boolean = await invoke("toggle_clipboard_pin", { id });
      items = items.map((i) => (i.id === id ? { ...i, pinned } : i));
    } catch (e) {
      console.error("Pin toggle failed:", e);
    }
  }

  function badgeFor(ct: string): { label: string; cls: string } {
    switch (ct) {
      case "url": return { label: "URL", cls: "badge-url" };
      case "email": return { label: "Email", cls: "badge-email" };
      case "color": return { label: "Color", cls: "badge-color" };
      case "json": return { label: "JSON", cls: "badge-json" };
      case "code": return { label: "Code", cls: "badge-code" };
      case "path": return { label: "Path", cls: "badge-path" };
      default: return { label: "Text", cls: "badge-text" };
    }
  }

  function timeAgo(ts: string): string {
    const diff = Date.now() - new Date(ts).getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 1) return "Just now";
    if (mins < 60) return `${mins}m ago`;
    const hrs = Math.floor(mins / 60);
    if (hrs < 24) return `${hrs}h ago`;
    const days = Math.floor(hrs / 24);
    return `${days}d ago`;
  }

  function charCount(s: string): string {
    const chars = s.length;
    const words = s.trim().split(/\s+/).length;
    const lines = s.split("\n").length;
    return `${chars} chars · ${words} words · ${lines} lines`;
  }

  function isColorValue(s: string): boolean {
    const t = s.trim();
    return (
      /^#[0-9a-fA-F]{3,8}$/.test(t) ||
      t.startsWith("rgb(") ||
      t.startsWith("rgba(") ||
      t.startsWith("hsl(")
    );
  }

  function isSensitiveValue(content: string): boolean {
    const trimmed = content.trim();
    if (/api[_-]?key/i.test(trimmed)) return true;
    if (/password\s*[:=]/i.test(trimmed)) return true;
    if (/token\s*[:=]/i.test(trimmed)) return true;
    if (/-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----/.test(trimmed)) return true;
    if (/gh[pousr]_[A-Za-z0-9_]{20,}/.test(trimmed)) return true;
    if (/AKIA[0-9A-Z]{16}/.test(trimmed)) return true;
    return false;
  }

  function previewText(item: ClipboardItem): string {
    if (maskSensitive && isSensitiveValue(item.content)) {
      return "Sensitive content hidden";
    }
    return item.content.replace(/\n/g, " ").substring(0, 80);
  }

  function detailText(item: ClipboardItem): string {
    if (maskSensitive && isSensitiveValue(item.content) && revealSensitiveId !== item.id) {
      return "Sensitive content hidden. Press Reveal to view.";
    }
    return item.content;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="cv-root">
  <div class="cv-toolbar">
    <button class="cv-toggle" class:active={dedupeEnabled} onclick={() => { dedupeEnabled = !dedupeEnabled; persistToggles(); }}>
      Dedupe
    </button>
    <button class="cv-toggle" class:active={groupByType} onclick={() => { groupByType = !groupByType; persistToggles(); }}>
      Group by Type
    </button>
    <button class="cv-toggle" class:active={maskSensitive} onclick={() => { maskSensitive = !maskSensitive; persistToggles(); }}>
      Mask Sensitive
    </button>
  </div>

  <div class="cv-search">
    <svg class="cv-search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="11" cy="11" r="8" /><path d="m21 21-4.3-4.3" />
    </svg>
    <!-- svelte-ignore a11y_autofocus -->
    <input
      class="cv-search-input"
      type="text"
      placeholder="Filter clipboard..."
      bind:value={query}
      autofocus
    />
    <span class="cv-count">{visibleItems.length}</span>
  </div>

  <div class="cv-body">
    <div class="cv-list" bind:this={listEl}>
      {#each grouped as block (block.label)}
        <div class="cv-group-header">{block.label}</div>
        {#each block.items as item (item.id)}
          {@const badge = badgeFor(item.content_type)}
          <button
            class="cv-item"
            class:cv-item-selected={item.id === selectedId}
            class:cv-item-pinned={item.pinned}
            onclick={() => (selectedId = item.id)}
            ondblclick={() => copyToClipboard(item.content)}
          >
            <div class="cv-item-top">
              {#if item.pinned}
                <span class="cv-pin-icon" title="Pinned">📌</span>
              {/if}
              <span class="cv-item-badge {badge.cls}">{badge.label}</span>
              <span class="cv-item-time">{timeAgo(item.timestamp)}</span>
            </div>
            <div class="cv-item-preview">
              {previewText(item)}
            </div>
          </button>
        {/each}
      {/each}
      {#if visibleItems.length === 0}
        <div class="cv-empty">No clipboard entries</div>
      {/if}
    </div>

    <div class="cv-detail">
      {#if selected}
        <div class="cv-detail-header">
          <span class="cv-detail-badge {badgeFor(selected.content_type).cls}">
            {badgeFor(selected.content_type).label}
          </span>
          <span class="cv-detail-meta">{charCount(selected.content)}</span>
        </div>

        {#if isColorValue(selected.content.trim())}
          <div class="cv-color-preview">
            <div class="cv-color-swatch" style:background={selected.content.trim()}></div>
            <span class="cv-color-label">{selected.content.trim()}</span>
          </div>
        {/if}

        <div class="cv-detail-content" class:cv-detail-code={selected.content_type === "code" || selected.content_type === "json"}>
          <pre>{detailText(selected)}</pre>
        </div>

        <div class="cv-detail-actions">
          <button class="cv-action-btn cv-action-copy" onclick={() => copyToClipboard(selected!.content)}>
            Copy
          </button>
          <button class="cv-action-btn cv-action-pin" onclick={() => togglePin(selected!.id)}>
            {selected.pinned ? "Unpin" : "Pin"}
          </button>
          {#if maskSensitive && isSensitiveValue(selected.content) && revealSensitiveId !== selected.id}
            <button class="cv-action-btn" onclick={() => (revealSensitiveId = selected.id)}>
              Reveal
            </button>
          {/if}
          <button class="cv-action-btn cv-action-delete" onclick={() => deleteItem(selected!.id)}>
            Delete
          </button>
        </div>
      {:else}
        <div class="cv-detail-empty">Select an item to preview</div>
      {/if}
    </div>
  </div>
</div>

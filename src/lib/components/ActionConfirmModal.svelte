<script lang="ts">
  import { onMount } from "svelte";

  let {
    title,
    description,
    confirmLabel = "Confirm",
    cancelLabel = "Cancel",
    onConfirm,
    onCancel,
    busy = false,
  }: {
    title: string;
    description: string;
    confirmLabel?: string;
    cancelLabel?: string;
    onConfirm: () => void;
    onCancel: () => void;
    busy?: boolean;
  } = $props();

  let primaryBtn: HTMLButtonElement | null = null;

  onMount(() => {
    primaryBtn?.focus();
  });

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      if (!busy) onCancel();
      return;
    }

    if (event.key === "Enter") {
      event.preventDefault();
      if (!busy) onConfirm();
    }
  }
</script>

<div class="action-confirm-overlay" role="presentation" onkeydown={onKeydown}>
  <div class="action-confirm-modal" role="dialog" aria-modal="true" aria-label={title}>
    <h3>{title}</h3>
    <p>{description}</p>
    <div class="action-confirm-actions">
      <button class="btn ghost" onclick={onCancel} disabled={busy}>{cancelLabel}</button>
      <button class="btn danger" onclick={onConfirm} disabled={busy} bind:this={primaryBtn}>
        {confirmLabel}
      </button>
    </div>
  </div>
</div>

<script lang="ts">
  import { toastHistory } from "$lib/stores/toastStore";

  let {
    onClose,
  }: {
    onClose: () => void;
  } = $props();

  function formatTime(epochMs: number): string {
    try {
      return new Date(epochMs).toLocaleTimeString();
    } catch {
      return "";
    }
  }
</script>

<div class="notification-overlay" role="dialog" aria-modal="true" aria-label="Notification center">
  <div class="notification-panel v2-panel">
    <div class="notification-header">
      <h3>Notifications</h3>
      <button class="close-btn" onclick={onClose} aria-label="Close notifications">×</button>
    </div>

    {#if $toastHistory.length === 0}
      <div class="notification-empty">No recent notifications.</div>
    {:else}
      <div class="notification-list">
        {#each $toastHistory as item (item.id)}
          <div class={`notification-item ${item.type}`}>
            <div class="notification-main">
              <strong>{item.title}</strong>
              {#if item.message}
                <span>{item.message}</span>
              {/if}
            </div>
            <time>{formatTime(item.createdAt)}</time>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

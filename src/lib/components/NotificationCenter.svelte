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
  <div class="notification-panel">
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

<style>
  .notification-overlay {
    position: fixed;
    inset: 0;
    z-index: 9990;
    display: flex;
    justify-content: flex-end;
    align-items: flex-end;
    background: rgba(0, 0, 0, 0.28);
  }

  .notification-panel {
    width: min(440px, calc(100vw - 24px));
    max-height: min(70vh, 680px);
    margin: 12px;
    border-radius: 14px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(12, 12, 18, 0.96);
    backdrop-filter: blur(10px);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .notification-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }

  .notification-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 700;
  }

  .close-btn {
    border: none;
    background: transparent;
    color: rgba(255, 255, 255, 0.8);
    width: 20px;
    height: 20px;
    cursor: pointer;
    font-size: 16px;
    line-height: 20px;
    padding: 0;
  }

  .notification-list {
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px;
  }

  .notification-item {
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 10px;
    padding: 8px 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .notification-item.success { border-color: rgba(52, 211, 153, 0.5); }
  .notification-item.error { border-color: rgba(248, 113, 113, 0.55); }
  .notification-item.info { border-color: rgba(96, 165, 250, 0.5); }

  .notification-main {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }

  .notification-main strong {
    font-size: 12px;
  }

  .notification-main span {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.78);
    overflow-wrap: anywhere;
  }

  .notification-item time {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.55);
    align-self: flex-end;
  }

  .notification-empty {
    padding: 16px;
    color: rgba(255, 255, 255, 0.68);
    font-size: 12px;
  }
</style>

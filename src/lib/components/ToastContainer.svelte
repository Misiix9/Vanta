<script lang="ts">
  import { toasts, dismissToast } from "$lib/stores/toastStore";

  function onDismiss(id: number) {
    dismissToast(id);
  }
</script>

<div class="toast-stack" aria-live="polite" aria-atomic="false">
  {#each $toasts as toast (toast.id)}
    <div class={`toast-item ${toast.type}`} role={toast.type === "error" ? "alert" : "status"}>
      <div class="toast-content">
        <strong>{toast.title}</strong>
        {#if toast.message}
          <span>{toast.message}</span>
        {/if}
      </div>
      <button class="toast-close" onclick={() => onDismiss(toast.id)} aria-label="Dismiss notification">×</button>
    </div>
  {/each}
</div>

<style>
  .toast-stack {
    position: fixed;
    right: 14px;
    bottom: 16px;
    z-index: 10000;
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: min(360px, calc(100vw - 20px));
    pointer-events: none;
  }

  .toast-item {
    pointer-events: auto;
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 10px;
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    background: rgba(16, 16, 20, 0.92);
    color: rgba(255, 255, 255, 0.95);
    box-shadow: 0 12px 30px rgba(0, 0, 0, 0.35);
    padding: 10px 12px;
    animation: toast-enter 180ms ease-out;
    backdrop-filter: blur(8px);
  }

  .toast-item.success { border-color: rgba(52, 211, 153, 0.5); }
  .toast-item.error { border-color: rgba(248, 113, 113, 0.6); }
  .toast-item.info { border-color: rgba(96, 165, 250, 0.5); }

  .toast-content {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .toast-content strong {
    font-size: 13px;
    line-height: 1.2;
  }

  .toast-content span {
    font-size: 12px;
    line-height: 1.3;
    color: rgba(255, 255, 255, 0.82);
    overflow-wrap: anywhere;
  }

  .toast-close {
    border: none;
    background: transparent;
    color: rgba(255, 255, 255, 0.72);
    cursor: pointer;
    padding: 0;
    width: 18px;
    height: 18px;
    line-height: 18px;
    font-size: 16px;
    flex-shrink: 0;
  }

  .toast-close:hover {
    color: #fff;
  }

  @keyframes toast-enter {
    from {
      transform: translateY(8px);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .toast-item {
      animation: none;
    }
  }
</style>

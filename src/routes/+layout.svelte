<script lang="ts">
  import '@fortawesome/fontawesome-free/css/all.min.css';

  function handleError(error: unknown) {
    console.error('[Vanta] Uncaught UI error:', error);
  }
</script>

<svelte:boundary onerror={handleError}>
  <slot />
  {#snippet failed(error, reset)}
    <div class="error-boundary">
      <p>Something went wrong.</p>
      <button onclick={reset}>Reload</button>
    </div>
  {/snippet}
</svelte:boundary>

<style>
  .error-boundary {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100vh;
    gap: 12px;
    font-family: var(--font-family, system-ui, sans-serif);
    color: var(--text-primary, #e0e0e0);
    background: var(--bg-primary, #1a1a2e);
  }
  .error-boundary button {
    padding: 6px 16px;
    border-radius: 6px;
    border: 1px solid var(--border-color, #444);
    background: var(--bg-secondary, #252540);
    color: var(--text-primary, #e0e0e0);
    cursor: pointer;
    font-size: 0.9rem;
  }
  .error-boundary button:hover {
    background: var(--bg-hover, #2f2f50);
  }
</style>

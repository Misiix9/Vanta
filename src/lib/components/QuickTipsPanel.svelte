<script lang="ts">
  let {
    onTry,
    onDismiss,
  }: {
    onTry: (query: string) => void;
    onDismiss: () => void;
  } = $props();

  const tips = [
    "weather",
    "timer 15m",
    "store",
    "restart",
    "calc (24*7)-5",
  ];
</script>

<section class="quick-tips" aria-label="Quick tips">
  <div class="quick-tips-header">
    <h4>Try These Commands</h4>
    <button class="link-btn" onclick={onDismiss}>Hide</button>
  </div>
  <p class="quick-tips-copy">Pick one to seed your search instantly.</p>
  <div class="quick-tips-row">
    {#each tips as tip}
      <button class="chip-btn" onclick={() => onTry(tip)}>{tip}</button>
    {/each}
  </div>
</section>

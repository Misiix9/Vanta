<script lang="ts">
  let {
    label,
    count,
    element = $bindable(undefined as HTMLDivElement | null | undefined),
  }: {
    label: string;
    count: number;
    element?: HTMLDivElement | null | undefined;
  } = $props();
</script>

<div
  class="group-header"
  bind:this={element}
  id={`section-${label}`}
  role="presentation"
  aria-hidden="false"
>
  <span class="group-title">
    {label}
  </span>
  <span class="group-count">{count}</span>
</div>

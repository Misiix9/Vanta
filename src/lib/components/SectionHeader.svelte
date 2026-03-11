<script lang="ts">
  let {
    label,
    count,
    isSelected = false,
    element = $bindable(undefined as HTMLDivElement | null | undefined),
  }: {
    label: string;
    count: number;
    isSelected?: boolean;
    element?: HTMLDivElement | null | undefined;
  } = $props();
</script>

<div
  class="group-header"
  class:selected={isSelected}
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

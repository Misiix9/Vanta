<script lang="ts">
  import { createEventDispatcher } from "svelte";

  let {
    label,
    count,
    collapsed = false,
    selected = false,
    element = $bindable(undefined as HTMLDivElement | null | undefined),
  }: {
    label: string;
    count: number;
    collapsed?: boolean;
    selected?: boolean;
    element?: HTMLDivElement | null | undefined;
  } = $props();

  const dispatch = createEventDispatcher<{ toggle: void; hover: void }>();

  function handleToggle() {
    dispatch("toggle");
  }
</script>

<div
  class="group-header"
  class:selected={selected}
  bind:this={element}
  id={`section-${label}`}
  role="button"
  tabindex="0"
  aria-expanded={!collapsed}
  aria-hidden="false"
  onmouseenter={() => dispatch("hover")}
  onclick={handleToggle}
  onkeydown={(e) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      handleToggle();
    }
  }}
>
  <span class="group-title">
    <span class={`chevron ${collapsed ? "collapsed" : ""}`}>▸</span>
    {label}
  </span>
  <span class="group-count">{count}</span>
</div>

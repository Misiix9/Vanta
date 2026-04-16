<script lang="ts">
  import type { SearchResult, ExtensionEntry, Capability, CommandContract } from "$lib/types";

  let {
    result,
    availableExtensions = [],
    onActivate,
    onAction,
    onQuickLook,
  }: {
    result: SearchResult | null;
    availableExtensions?: ExtensionEntry[];
    onActivate?: (result: SearchResult) => void;
    onAction?: (result: SearchResult, exec: string, command?: CommandContract) => void;
    onQuickLook?: (result: SearchResult) => void;
  } = $props();

  let actionButtons: HTMLButtonElement[] = [];

  export function focusFirstAction() {
    actionButtons[0]?.focus();
  }

  function sourceLabel(source: SearchResult["source"]): string {
    if (typeof source === "object") return `Extension (${source.Extension.ext_id})`;
    return source;
  }

  function getExtensionEntry(result: SearchResult): ExtensionEntry | null {
    if (typeof result.source !== "object") return null;
    const extId = result.source.Extension.ext_id;
    return availableExtensions.find((ext) => ext.manifest.name === extId) ?? null;
  }

  function formatPermissionCaps(caps: Capability[]): string {
    if (!caps.length) return "No elevated capabilities";
    return caps.join(" · ");
  }

  function safetySummary(result: SearchResult): string {
    const exec = result.exec.toLowerCase();
    if (exec.startsWith("system-action:shutdown") || exec.startsWith("system-action:restart") || exec.startsWith("system-action:logout") || exec.startsWith("system-action:bios")) {
      return "Sensitive action — confirmation required before execution.";
    }
    const ext = getExtensionEntry(result);
    if (ext) {
      const caps = ext.manifest.permissions ?? [];
      if (caps.includes("Shell") || caps.includes("Filesystem")) {
        return `Elevated extension permissions: ${formatPermissionCaps(caps)}`;
      }
      return `Extension permissions: ${formatPermissionCaps(caps)}`;
    }
    return "Standard action context.";
  }

  function templateTitle(result: SearchResult): string {
    if (typeof result.source === "object") return "Extension Panel";
    if (result.command?.kind === "macro_open") return "Workflow Panel";
    if (result.source === "File") return "File Panel";
    if (result.source === "Window") return "Window Panel";
    if (result.source === "Clipboard") return "Clipboard Panel";
    if (result.source === "Calculator") return "Calculator Panel";
    return "Application Panel";
  }
</script>

<aside class="context-panel v2-panel" aria-label="Contextual details panel">
  {#if result}
    <header class="context-panel-header">
      <h3>{result.title}</h3>
      <span class="context-type">{templateTitle(result)}</span>
    </header>

    <div class="context-panel-meta">
      <p>{result.subtitle ?? "No description available."}</p>
      <span class="v2-form-help">Source: {sourceLabel(result.source)}</span>
      <span class="v2-form-help">{safetySummary(result)}</span>
    </div>

    {#if result.source === "File"}
      <section class="context-section">
        <h4>File Context</h4>
        <p class="context-value">{result.exec.startsWith("open:") ? result.exec.slice(5) : (result.subtitle ?? result.exec)}</p>
      </section>
    {:else if result.source === "Window"}
      <section class="context-section">
        <h4>Window Context</h4>
        <p class="context-value">Use quick actions to focus, minimize, move to workspace, or close.</p>
      </section>
    {:else if result.source === "Clipboard"}
      <section class="context-section">
        <h4>Clipboard Context</h4>
        <p class="context-value">{result.subtitle ?? "Clipboard item"}</p>
      </section>
    {:else if typeof result.source === "object"}
      {@const ext = getExtensionEntry(result)}
      <section class="context-section">
        <h4>Extension Context</h4>
        <p class="context-value">{ext?.manifest.description ?? "Extension action"}</p>
        <span class="v2-form-help">Capabilities: {formatPermissionCaps(ext?.manifest.permissions ?? [])}</span>
      </section>
    {:else if result.command?.kind === "macro_open"}
      <section class="context-section">
        <h4>Workflow Context</h4>
        <p class="context-value">Macro workflows can be dry-run and executed with argument previews.</p>
      </section>
    {/if}

    <div class="context-actions">
      <button
        class="btn-secondary"
        bind:this={actionButtons[0]}
        onclick={() => onActivate?.(result)}
      >
        Open
      </button>
      <button class="btn-ghost" bind:this={actionButtons[1]} onclick={() => onQuickLook?.(result)}>
        Quick Look
      </button>
      {#if result.actions?.length}
        {#each result.actions as action, i}
          <button
            class="btn-ghost"
            bind:this={actionButtons[i + 2]}
            onclick={() => onAction?.(result, action.exec ?? result.exec, action.command)}
          >
            {action.label}
          </button>
        {/each}
      {/if}
    </div>
  {:else}
    <p class="context-empty">Select a result to view contextual details, quick actions, and safety cues.</p>
  {/if}
</aside>

<style>
  .context-panel {
    min-height: 0;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: calc(var(--space-2) * var(--vanta-space-scale, 1));
  }

  .context-panel-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: calc(var(--space-1) * var(--vanta-space-scale, 1));
  }

  .context-panel-header h3 {
    margin: 0;
    font-size: var(--type-heading);
    color: var(--ds-text-primary);
  }

  .context-type {
    font-size: var(--type-caption);
    color: var(--ds-text-muted);
  }

  .context-panel-meta {
    display: flex;
    flex-direction: column;
    gap: calc(var(--space-1) * var(--vanta-space-scale, 1));
  }

  .context-panel-meta p {
    margin: 0;
    color: var(--ds-text-secondary);
    font-size: var(--type-body);
    line-height: 1.45;
  }

  .context-section {
    display: flex;
    flex-direction: column;
    gap: calc(var(--space-1) * var(--vanta-space-scale, 1));
    padding: calc(var(--space-2) * var(--vanta-space-scale, 1));
    border: 1px solid color-mix(in srgb, var(--ds-border) 68%, transparent);
    border-radius: var(--radius-item);
    background: color-mix(in srgb, var(--ds-surface-2) 65%, transparent);
  }

  .context-section h4 {
    margin: 0;
    font-size: var(--type-caption);
    color: var(--ds-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .context-value {
    margin: 0;
    font-size: var(--type-body);
    color: var(--ds-text-primary);
    word-break: break-word;
  }

  .context-actions {
    display: flex;
    flex-direction: column;
    gap: calc(var(--space-1) * var(--vanta-space-scale, 1));
  }

  .context-empty {
    margin: 0;
    font-size: var(--type-body);
    color: var(--ds-text-muted);
  }
</style>

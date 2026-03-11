<script lang="ts">
  import type {
    WorkflowMacro,
    MacroDryRunResult,
    MacroDryRunStep,
    Capability,
  } from "$lib/types";

  let {
    macro,
    args,
    dryRun,
    busy = false,
    onArgsChange,
    onDryRun,
    onRun,
    onClose,
    error,
  }: {
    macro: WorkflowMacro;
    args: Record<string, string>;
    dryRun: MacroDryRunResult | null;
    busy?: boolean;
    error?: string | null;
    onArgsChange: (name: string, value: string) => void;
    onDryRun: () => void;
    onRun: () => void;
    onClose: () => void;
  } = $props();

  const decisionLabel: Record<string, string> = {
    Allow: "Allowed",
    Ask: "Needs permission",
    Deny: "Blocked",
  };

  function capabilityIcon(cap: Capability): string {
    switch (cap) {
      case "Network":
        return "🌐";
      case "Shell":
        return "💻";
      case "Filesystem":
        return "🗂️";
    }
  }
</script>

<div class="macro-panel">
  <header class="macro-header">
    <div>
      <p class="macro-eyebrow">Macro</p>
      <h2>{macro.name}</h2>
      {#if macro.description}
        <p class="macro-subtitle">{macro.description}</p>
      {/if}
    </div>
    <button class="ghost" onclick={onClose}>Close</button>
  </header>

  <section class="macro-args">
    <div class="args-header">
      <h3>Arguments</h3>
      {#if macro.args?.length === 0}
        <span class="muted">No arguments</span>
      {/if}
    </div>

    {#if macro.args?.length}
      <div class="args-grid">
        {#each macro.args as arg}
          <label class="arg-field">
            <span class="arg-label">{arg.name}{arg.required ? " *" : ""}</span>
            {#if arg.description}
              <span class="arg-help">{arg.description}</span>
            {/if}
            <input
              type="text"
              value={args[arg.name] ?? ""}
              placeholder={arg.default_value ?? ""}
              oninput={(e) => onArgsChange(arg.name, (e.target as HTMLInputElement).value)}
            />
          </label>
        {/each}
      </div>
    {/if}
  </section>

  <section class="macro-actions">
    <div class="left">
      <button class="secondary" onclick={onDryRun} disabled={busy}>Dry Run</button>
      <button
        class="primary"
        onclick={onRun}
        disabled={busy || (dryRun && dryRun.errors.length > 0)}
        title={dryRun && !dryRun.ready ? "Will prompt for permissions" : ""}
      >
        {busy
          ? "Running..."
          : dryRun && !dryRun.ready
            ? "Request Permission"
            : "Run Macro"}
      </button>
    </div>
    <div class="right">
      {#if dryRun}
        <span class={dryRun.ready ? "status-good" : "status-warn"}>
          {dryRun.ready ? "Ready" : "Blocked"}
        </span>
      {/if}
      {#if error}
        <span class="status-err">{error}</span>
      {/if}
    </div>
  </section>

  <section class="macro-steps">
    <div class="steps-header">
      <h3>Steps</h3>
      {#if !dryRun}
        <span class="muted">Run a dry-run to preview</span>
      {/if}
    </div>

    {#if dryRun}
      {#if dryRun.errors.length}
        <div class="error-box">
          {#each dryRun.errors as err}
            <div class="error-row">{err}</div>
          {/each}
        </div>
      {/if}

      <div class="steps-list">
        {#each dryRun.steps as step (step.index)}
          <div class="step-row">
            <div class="step-index">{step.index + 1}</div>
            <div class="step-body">
              <div class="step-title">{step.kind === "script" ? "Script" : "System"} · {step.command}</div>
              {#if step.args.length}
                <div class="step-args">{step.args.join(" ")}</div>
              {/if}
              {#if step.capabilities.length}
                <div class="caps">
                  {#each step.capabilities as cap}
                    <span class="cap-chip">{capabilityIcon(cap)} {cap}</span>
                  {/each}
                </div>
              {/if}
            </div>
            <div class={`step-status decision-${step.decision.toLowerCase()}`}>
              {decisionLabel[step.decision] ?? step.decision}
              {#if step.missing_caps.length}
                <small>Needs: {step.missing_caps.join(", ")}</small>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </section>
</div>

<style>
  .macro-panel {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 16px;
    height: 100%;
    overflow: auto;
  }
  .macro-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 12px;
  }
  .macro-eyebrow { font-size: var(--type-body); opacity: 0.6; margin: 0; line-height: var(--leading-normal); }
  .macro-subtitle { margin: 4px 0 0 0; opacity: 0.8; }
  .macro-args .args-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 12px; }
  .arg-field { display: flex; flex-direction: column; gap: 4px; padding: 8px; border: 1px solid rgba(255,255,255,0.06); border-radius: 10px; background: rgba(255,255,255,0.02); }
  .arg-label { font-weight: var(--weight-semibold); }
  .arg-help { font-size: var(--type-body); opacity: 0.7; line-height: var(--leading-normal); }
  .arg-field input { width: 100%; padding: 8px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.1); background: rgba(0,0,0,0.3); color: #fff; }
  .macro-actions { display: flex; justify-content: space-between; align-items: center; gap: 12px; flex-wrap: wrap; }
  .macro-actions .left { display: flex; gap: 8px; align-items: center; }
  .macro-actions .right { display: flex; gap: 8px; align-items: center; }
  button.primary { background: var(--vanta-accent, #7c6ff7); color: #fff; padding: 10px 14px; border-radius: 10px; border: none; font-weight: var(--weight-semibold); cursor: pointer; }
  button.secondary { background: transparent; color: #fff; padding: 10px 12px; border-radius: 10px; border: 1px solid rgba(255,255,255,0.2); cursor: pointer; }
  button.ghost { background: transparent; border: 1px solid rgba(255,255,255,0.2); color: #fff; padding: 8px 10px; border-radius: 10px; cursor: pointer; }
  button:disabled { opacity: 0.6; cursor: not-allowed; }
  .status-good { color: var(--ds-success); font-weight: var(--weight-semibold); }
  .status-warn { color: var(--ds-warning); font-weight: var(--weight-semibold); }
  .status-err { color: var(--ds-danger); font-weight: var(--weight-semibold); }
  .macro-steps { display: flex; flex-direction: column; gap: 12px; }
  .steps-header { display: flex; justify-content: space-between; align-items: center; }
  .steps-list { display: flex; flex-direction: column; gap: 10px; }
  .step-row { display: grid; grid-template-columns: 36px 1fr auto; gap: 12px; padding: 10px; border: 1px solid rgba(255,255,255,0.08); border-radius: 10px; background: rgba(255,255,255,0.02); }
  .step-index { width: 32px; height: 32px; border-radius: 8px; background: rgba(255,255,255,0.05); display: grid; place-items: center; font-weight: var(--weight-bold); }
  .step-title { font-weight: var(--weight-semibold); }
  .step-args { opacity: 0.8; font-family: var(--font-mono); font-size: var(--type-body); }
  .caps { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 6px; }
  .cap-chip { padding: 4px 6px; border-radius: 8px; background: rgba(255,255,255,0.06); font-size: var(--type-body); }
  .step-status { text-align: right; display: flex; flex-direction: column; gap: 4px; align-items: flex-end; font-weight: var(--weight-semibold); }
  .step-status small { opacity: 0.8; font-weight: var(--weight-normal); }
  .decision-allow { color: var(--ds-success); }
  .decision-ask { color: var(--ds-warning); }
  .decision-deny { color: var(--ds-danger); }
  .muted { opacity: 0.7; font-size: var(--type-body); line-height: var(--leading-normal); }
  .error-box { border: 1px solid color-mix(in srgb, var(--ds-danger) 30%, transparent); background: var(--ds-danger-muted); border-radius: 10px; padding: 8px; display: flex; flex-direction: column; gap: 6px; }
  .error-row { color: var(--ds-danger); font-size: var(--type-body); line-height: var(--leading-normal); }
</style>

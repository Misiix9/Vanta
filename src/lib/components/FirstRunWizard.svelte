<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import type { ThemeMeta } from "$lib/types";

  let {
    hotkey,
    themeId,
    includeHidden,
    themes = [],
  }: {
    hotkey: string;
    themeId: string;
    includeHidden: boolean;
    themes: ThemeMeta[];
  } = $props();

  const dispatch = createEventDispatcher<{
    tryquery: { query: string };
    sethotkey: { hotkey: string };
    settheme: { themeId: string };
    setincludehidden: { includeHidden: boolean };
    opensettings: undefined;
    complete: undefined;
    skip: undefined;
  }>();

  let step = $state(0);
  let modalEl: HTMLDivElement | null = $state(null);

  const samples = [
    "weather",
    "store",
    "feature hub",
    "community sharing",
    "extension template",
  ];

  onMount(() => {
    modalEl?.focus();
  });

  function nextStep() {
    if (step < 2) {
      step += 1;
    } else {
      dispatch("complete");
    }
  }

  function prevStep() {
    if (step > 0) step -= 1;
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      dispatch("skip");
      return;
    }

    if (event.key === "Enter" && !event.shiftKey) {
      const target = event.target as HTMLElement;
      if (target?.tagName !== "INPUT" && target?.tagName !== "SELECT") {
        event.preventDefault();
        nextStep();
      }
    }
  }
</script>

<div class="onboarding-overlay" role="presentation" onkeydown={onKeydown}>
  <div
    class="onboarding-modal"
    role="dialog"
    aria-modal="true"
    aria-label="First run onboarding"
    tabindex="-1"
    bind:this={modalEl}
  >
    <div class="onboarding-header">
      <div class="pill">First Run</div>
      <div class="onboarding-progress">Step {step + 1} / 3</div>
    </div>

    {#if step === 0}
      <h3>Welcome to Vanta</h3>
      <p class="onboarding-copy">
        Start with a few commands to learn the flow quickly. Type, arrow through results, and hit Enter.
      </p>
      <div class="onboarding-chip-row">
        {#each samples as sample}
          <button class="chip-btn" onclick={() => dispatch("tryquery", { query: sample })}>
            {sample}
          </button>
        {/each}
      </div>
      <p class="onboarding-copy">
        Power shortcuts: <code>Ctrl+,</code> opens Settings, <code>Shift+Enter</code> reveals file path, and <code>Alt+Enter</code> opens with your editor.
      </p>
    {:else if step === 1}
      <h3>Personalize Essentials</h3>
      <p class="onboarding-copy">Set your preferred launcher hotkey and initial theme. You can refine everything later in Settings.</p>
      <div class="onboarding-grid">
        <label>
          Hotkey
          <input
            type="text"
            value={hotkey}
            oninput={(e) => dispatch("sethotkey", { hotkey: (e.target as HTMLInputElement).value })}
          />
        </label>
        <label>
          Theme
          <select
            class="vanta-select"
            value={themeId}
            onchange={(e) => dispatch("settheme", { themeId: (e.target as HTMLSelectElement).value })}
          >
            {#each themes as theme}
              <option value={theme.id}>{theme.name}</option>
            {/each}
          </select>
        </label>
      </div>
    {:else}
      <h3>Search And Permissions</h3>
      <p class="onboarding-copy">
        Choose your default file-search scope and review how extension permissions are requested.
      </p>
      <label class="inline-toggle">
        <input
          type="checkbox"
          checked={includeHidden}
          onchange={(e) =>
            dispatch("setincludehidden", {
              includeHidden: (e.target as HTMLInputElement).checked,
            })}
        />
        Include hidden files in search
      </label>

      <div class="onboarding-permission-note">
        <strong>Permission guidance:</strong>
        <ul>
          <li><code>Network</code>: web/API access only when needed.</li>
          <li><code>Filesystem</code>: local file read/write actions.</li>
          <li><code>Shell</code>: command execution; use for trusted extensions only.</li>
        </ul>
      </div>

      <button class="link-btn" onclick={() => dispatch("opensettings")}>Open full settings</button>
    {/if}

    <div class="onboarding-actions">
      <button class="btn ghost" onclick={() => dispatch("skip")}>Skip</button>
      {#if step > 0}
        <button class="btn ghost" onclick={prevStep}>Back</button>
      {/if}
      <button class="btn primary" onclick={nextStep}>{step < 2 ? "Next" : "Finish"}</button>
    </div>
  </div>
</div>

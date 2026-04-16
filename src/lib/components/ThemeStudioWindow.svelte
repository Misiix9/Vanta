<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { VantaConfig, ThemeMeta } from "$lib/types";
  import { applyTheme } from "$lib/theme";

  let {
    config = $bindable(),
    availableThemes = [],
    onClose,
  }: {
    config: VantaConfig;
    availableThemes: ThemeMeta[];
    onClose: () => void;
  } = $props();

  let text = $state("");
  let status = $state<string | null>(null);

  const ADAPTIVE_PROFILE_OPTIONS = ["balanced", "daylight", "night", "presentation"] as const;
  const ADAPTIVE_LIGHTING_OPTIONS = ["neutral", "bright", "dim"] as const;
  const ADAPTIVE_DENSITY_OPTIONS = ["compact", "comfortable", "relaxed"] as const;
  const ADAPTIVE_PERFORMANCE_OPTIONS = ["quality", "balanced", "battery"] as const;
  const ADAPTIVE_ACCESSIBILITY_OPTIONS = ["inherit", "readability", "focus", "calm"] as const;

  function normalizeAdaptiveOption<T extends readonly string[]>(value: unknown, allowed: T, fallback: T[number]): T[number] {
    return typeof value === "string" && (allowed as readonly string[]).includes(value) ? (value as T[number]) : fallback;
  }

  function ensureAdaptiveProfile() {
    const adaptive = config.appearance.adaptive;
    if (!adaptive || typeof adaptive !== "object") {
      config.appearance.adaptive = {
        enabled: false,
        profile: "balanced",
        lighting: "neutral",
        density: "comfortable",
        performance_tier: "balanced",
        accessibility_preset: "inherit",
      };
      return;
    }

    config.appearance.adaptive.enabled = Boolean(adaptive.enabled);
    config.appearance.adaptive.profile = normalizeAdaptiveOption(adaptive.profile, ADAPTIVE_PROFILE_OPTIONS, "balanced");
    config.appearance.adaptive.lighting = normalizeAdaptiveOption(adaptive.lighting, ADAPTIVE_LIGHTING_OPTIONS, "neutral");
    config.appearance.adaptive.density = normalizeAdaptiveOption(adaptive.density, ADAPTIVE_DENSITY_OPTIONS, "comfortable");
    config.appearance.adaptive.performance_tier = normalizeAdaptiveOption(
      adaptive.performance_tier,
      ADAPTIVE_PERFORMANCE_OPTIONS,
      "balanced",
    );
    config.appearance.adaptive.accessibility_preset = normalizeAdaptiveOption(
      adaptive.accessibility_preset,
      ADAPTIVE_ACCESSIBILITY_OPTIONS,
      "inherit",
    );
  }

  ensureAdaptiveProfile();

  function saveConfig() {
    applyTheme(config);
    invoke("save_config", { newConfig: config }).catch((e) => {
      status = `Save failed: ${String(e)}`;
    });
  }

  function applyPreset(kind: "minimal" | "vivid" | "high-contrast") {
    if (kind === "minimal") {
      config.appearance.colors.background = "#070707";
      config.appearance.colors.surface = "#111111";
      config.appearance.colors.accent = "#DADADA";
      config.appearance.colors.accent_glow = "rgba(218, 218, 218, 0.22)";
      config.appearance.colors.text_primary = "#F2F2F2";
      config.appearance.colors.text_secondary = "#A0A0A0";
      config.appearance.colors.border = "rgba(255, 255, 255, 0.1)";
    } else if (kind === "vivid") {
      config.appearance.colors.background = "#0B1020";
      config.appearance.colors.surface = "#151B33";
      config.appearance.colors.accent = "#4DE3C3";
      config.appearance.colors.accent_glow = "rgba(77, 227, 195, 0.35)";
      config.appearance.colors.text_primary = "#ECF5FF";
      config.appearance.colors.text_secondary = "#92A2C5";
      config.appearance.colors.border = "rgba(77, 227, 195, 0.22)";
    } else {
      config.appearance.colors.background = "#000000";
      config.appearance.colors.surface = "#0A0A0A";
      config.appearance.colors.accent = "#FFD400";
      config.appearance.colors.accent_glow = "rgba(255, 212, 0, 0.32)";
      config.appearance.colors.text_primary = "#FFFFFF";
      config.appearance.colors.text_secondary = "#C2C2C2";
      config.appearance.colors.border = "rgba(255, 255, 255, 0.2)";
    }
    status = `Applied ${kind} preset.`;
    saveConfig();
  }

  function exportProfile() {
    text = JSON.stringify({ version: 1, appearance: config.appearance, window: config.window }, null, 2);
    status = "Theme profile exported.";
  }

  function importProfile() {
    try {
      const parsed = JSON.parse(text);
      if (!parsed?.appearance?.colors) {
        status = "Invalid payload: missing appearance colors.";
        return;
      }
      config.appearance = parsed.appearance;
      ensureAdaptiveProfile();
      if (parsed.window) config.window = parsed.window;
      status = "Theme profile imported.";
      saveConfig();
    } catch {
      status = "Invalid JSON payload.";
    }
  }
</script>

<div class="v2-shell v2-fill hub-window-root">
  <div class="v2-header hub-window-header">
    <div class="window-header-main">
      <span class="window-breadcrumb">Feature Hub</span>
      <h2>Theme And Profile Studio</h2>
    </div>
    <div class="window-actions">
      <button class="link-btn btn-ghost" onclick={onClose}>Back</button>
      <button class="link-btn btn-secondary" onclick={saveConfig}>Save</button>
    </div>
  </div>

  <div class="v2-scroll window-scroll">
    <div class="control-group">
      <label>
        Active Theme
        <select class="vanta-select" bind:value={config.appearance.theme} onchange={saveConfig}>
          {#each availableThemes as theme}
            <option value={theme.id}>{theme.name}</option>
          {/each}
        </select>
      </label>
    </div>

    <div class="preset-row">
      <button class="preset-btn" onclick={() => applyPreset("minimal")}>Minimal</button>
      <button class="preset-btn" onclick={() => applyPreset("vivid")}>Vivid</button>
      <button class="preset-btn" onclick={() => applyPreset("high-contrast")}>High Contrast</button>
    </div>

    <div class="control-group">
      <label>Background <input type="color" bind:value={config.appearance.colors.background} oninput={saveConfig} /></label>
      <label>Surface <input type="color" bind:value={config.appearance.colors.surface} oninput={saveConfig} /></label>
      <label>Accent <input type="color" bind:value={config.appearance.colors.accent} oninput={saveConfig} /></label>
      <label>Opacity <input type="range" min="0.1" max="1" step="0.05" bind:value={config.appearance.opacity} oninput={saveConfig} /></label>
    </div>

    <div class="control-group control-group-block">
      <h3>Adaptive Appearance</h3>
      <label>
        <input type="checkbox" bind:checked={config.appearance.adaptive.enabled} onchange={saveConfig} />
        Enable adaptive profile transforms
      </label>
      <label>
        Profile
        <select class="vanta-select" bind:value={config.appearance.adaptive.profile} onchange={saveConfig}>
          {#each ADAPTIVE_PROFILE_OPTIONS as profile}
            <option value={profile}>{profile}</option>
          {/each}
        </select>
      </label>
      <label>
        Lighting
        <select class="vanta-select" bind:value={config.appearance.adaptive.lighting} onchange={saveConfig}>
          {#each ADAPTIVE_LIGHTING_OPTIONS as lighting}
            <option value={lighting}>{lighting}</option>
          {/each}
        </select>
      </label>
      <label>
        Density
        <select class="vanta-select" bind:value={config.appearance.adaptive.density} onchange={saveConfig}>
          {#each ADAPTIVE_DENSITY_OPTIONS as density}
            <option value={density}>{density}</option>
          {/each}
        </select>
      </label>
      <label>
        Performance Tier
        <select class="vanta-select" bind:value={config.appearance.adaptive.performance_tier} onchange={saveConfig}>
          {#each ADAPTIVE_PERFORMANCE_OPTIONS as tier}
            <option value={tier}>{tier}</option>
          {/each}
        </select>
      </label>
      <label>
        Accessibility Preset
        <select class="vanta-select" bind:value={config.appearance.adaptive.accessibility_preset} onchange={saveConfig}>
          {#each ADAPTIVE_ACCESSIBILITY_OPTIONS as preset}
            <option value={preset}>{preset}</option>
          {/each}
        </select>
      </label>
    </div>

    <div class="control-group control-group-block">
      <h3>Theme Profile JSON</h3>
      <div class="preset-row">
        <button class="preset-btn" onclick={exportProfile}>Export</button>
        <button class="preset-btn" onclick={importProfile}>Import</button>
      </div>
      <textarea rows="10" bind:value={text}></textarea>
    </div>

    {#if status}
      <div class="status-info">{status}</div>
    {/if}
  </div>
</div>

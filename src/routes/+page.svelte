<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { onDestroy, onMount } from "svelte";
  import { fade } from "svelte/transition";
  import type { ToastOptions } from "$lib/sdk/types";
  import type {
    VantaConfig,
    BlurStatus,
    ThemeMeta,
    Capability,
    WorkflowMacro,
    ExtensionEntry,
  } from "$lib/types";
  import { applyTheme, validateThemeTokens } from "$lib/theme";
  import SettingsView from "$lib/components/SettingsView.svelte";
  import PermissionModal from "$lib/components/PermissionModal.svelte";
  import StoreView from "$lib/components/StoreView.svelte";
  import ClipboardView from "$lib/components/ClipboardView.svelte";
  import FeatureHubWindow from "$lib/components/FeatureHubWindow.svelte";
  import CommunityHubWindow from "$lib/components/CommunityHubWindow.svelte";
  import ThemeStudioWindow from "$lib/components/ThemeStudioWindow.svelte";
  import ExtensionsHubWindow from "$lib/components/ExtensionsHubWindow.svelte";
  import SpotifyMiniPlayer from "$lib/components/SpotifyMiniPlayer.svelte";
  import ExtensionHost from "$lib/components/ExtensionHost.svelte";
  import LauncherView from "$lib/components/LauncherView.svelte";
  import ToastContainer from "$lib/components/ToastContainer.svelte";
  import { addToast } from "$lib/stores/toastStore";

  type ViewId = "launcher" | "settings" | "store" | "featureHub" | "communityHub" | "themeHub" | "extensionsHub";
  interface UiStateSnapshot {
    last_view?: string | null;
  }

  let isMiniPlayer = $state(false);
  let vantaConfig: VantaConfig | undefined = $state();
  let view: ViewId = $state("launcher");
  let currentMode: "launcher" | "clipboard" = $state("launcher");
  let settingsStartSection = $state<string | null>(null);
  let blurMode: string = $state("fallback");
  let availableExtensions: ExtensionEntry[] = $state([]);
  let extensionView: { extId: string; command: string; extPath: string } | null = $state(null);
  let availableMacros: WorkflowMacro[] = $state([]);
  let isVisible = $state(true);
  let permissionPrompt = $state<{
    scriptId: string;
    missingCaps: Capability[];
    requestedCaps: Capability[];
  } | null>(null);
  let permissionBusy = $state(false);
  let availableThemes: ThemeMeta[] = $state([]);
  let clipboardHistory: any[] = $state([]);
  let launcherRef: LauncherView | undefined = $state();

  const viewLabels: Record<ViewId, string> = {
    launcher: "Search",
    settings: "Settings",
    store: "Vanta Store",
    featureHub: "Feature Hub",
    communityHub: "Community Hub",
    themeHub: "Theme Studio",
    extensionsHub: "Extensions Hub",
  };
  const viewParents: Partial<Record<ViewId, ViewId>> = {
    communityHub: "featureHub",
    themeHub: "featureHub",
    extensionsHub: "featureHub",
  };

  let viewHistory = $state<ViewId[]>([]);
  let viewHistoryCursor = $state<ViewId>("launcher");
  let suppressViewHistoryPush = false;

  function isEditableTarget(target: EventTarget | null): boolean {
    const el = target as HTMLElement | null;
    if (!el) return false;
    if (el.isContentEditable) return true;
    const tag = el.tagName;
    return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT";
  }

  function buildViewTrail(current: ViewId): ViewId[] {
    const trail: ViewId[] = [current];
    let parent = viewParents[current];
    while (parent) {
      trail.unshift(parent);
      parent = viewParents[parent];
    }
    if (trail[0] !== "launcher") {
      trail.unshift("launcher");
    }
    return trail;
  }

  function focusLauncherSurface() {
    if (currentMode !== "launcher") {
      currentMode = "launcher";
    }
    requestAnimationFrame(() => {
      launcherRef?.refreshSuggestions();
      launcherRef?.focus();
    });
  }

  function goBackInNavigation() {
    if (viewHistory.length > 0) {
      const previous = viewHistory[viewHistory.length - 1];
      viewHistory = viewHistory.slice(0, -1);
      suppressViewHistoryPush = true;
      view = previous;
      if (previous === "launcher") {
        settingsStartSection = null;
        focusLauncherSurface();
      }
      return;
    }

    if (view !== "launcher") {
      view = "launcher";
      settingsStartSection = null;
      focusLauncherSurface();
    }
  }

  function goHomeInNavigation() {
    if (view !== "launcher") {
      view = "launcher";
      settingsStartSection = null;
      focusLauncherSurface();
    }
  }

  function handleGlobalWayfindingKeydown(event: KeyboardEvent) {
    if (isMiniPlayer || isEditableTarget(event.target)) return;
    if (event.defaultPrevented) return;

    const key = event.key.toLowerCase();
    if (event.altKey && !event.ctrlKey && !event.metaKey && key === "arrowleft") {
      event.preventDefault();
      event.stopPropagation();
      goBackInNavigation();
      return;
    }

    if (event.altKey && !event.ctrlKey && !event.metaKey && (key === "home" || key === "h")) {
      event.preventDefault();
      event.stopPropagation();
      goHomeInNavigation();
    }
  }

  let breadcrumbTrail = $derived.by(() => buildViewTrail(view));
  let lastReturnView = $derived.by(() =>
    viewHistory.length > 0 ? viewHistory[viewHistory.length - 1] : null,
  );
  let canGoBack = $derived.by(() => view !== "launcher" || viewHistory.length > 0);
  let wayfindingNextStep = $derived.by(() => {
    if (view === "featureHub") return "Open Community, Themes, Extensions, or Store";
    if (view === "communityHub") return "Import trusted artifacts or rollback recent imports";
    if (view === "themeHub") return "Save presets or export profile JSON";
    if (view === "extensionsHub") return "Create template or open Store";
    if (view === "store") return "Install or update extensions";
    if (view === "settings") return "Adjust preferences, then return to Search";
    return null;
  });
  let wayfindingNextAction = $derived.by(() => {
    if (view === "settings") return { label: "Open Store", target: "store" as ViewId };
    if (view === "store") return { label: "Feature Hub", target: "featureHub" as ViewId };
    if (view === "featureHub") return { label: "Community", target: "communityHub" as ViewId };
    if (view === "communityHub") return { label: "Theme Studio", target: "themeHub" as ViewId };
    if (view === "themeHub") return { label: "Extensions Hub", target: "extensionsHub" as ViewId };
    if (view === "extensionsHub") return { label: "Feature Hub", target: "featureHub" as ViewId };
    return null;
  });
  let viewAnnouncement = $derived(viewLabels[view] ?? "");

  const RESCAN_INTERVAL_MS = 15000;
  let lastAppRescan = $state(0);
  const unlisteners: Array<() => void> = [];
  let fadeDuration = $derived(vantaConfig?.accessibility?.reduced_motion ? 0 : 150);

  const validViews = new Set<ViewId>([
    "launcher",
    "settings",
    "store",
    "featureHub",
    "communityHub",
    "themeHub",
    "extensionsHub",
  ]);

  function asViewId(value: string | null | undefined): ViewId | null {
    if (!value) return null;
    return validViews.has(value as ViewId) ? (value as ViewId) : null;
  }

  function notify(options: ToastOptions) {
    addToast(options);
  }

  async function maybeRescanApps(force = false) {
    const now = Date.now();
    if (!force && now - lastAppRescan < RESCAN_INTERVAL_MS) return;
    lastAppRescan = now;
    try { await invoke<number>("rescan_apps"); }
    catch (e) {
      console.error("App rescan failed", e);
      notify({ title: "Rescan Failed", message: String(e), type: "error" });
    }
  }

  function injectThemeCss(css: string) {
    let el = document.getElementById("vanta-theme") as HTMLStyleElement | null;
    if (!el) {
      el = document.createElement("style");
      el.id = "vanta-theme";
      document.head.appendChild(el);
    }
    el.textContent = css;
  }

  function applyThemeFromConfig(config: VantaConfig) {
    applyTheme(config);
    const themeId = config.appearance.theme || "default";
    const target =
      availableThemes.find((t) => t.id === themeId) ||
      availableThemes.find((t) => t.id === "default");
    if (target) {
      injectThemeCss(target.css_content);
      for (const d of validateThemeTokens(target.css_content)) {
        if (d.level === "error") console.warn(`[theme] ${d.token}: ${d.message}`);
        else console.info(`[theme] ${d.token}: ${d.message}`);
      }
    }
  }

  async function saveConfigUpdate(mutator: (cfg: VantaConfig) => void) {
    if (!vantaConfig) return;
    mutator(vantaConfig);
    applyThemeFromConfig(vantaConfig);
    try { await invoke("save_config", { newConfig: vantaConfig }); }
    catch (e) {
      console.error("Failed to save config", e);
      notify({ title: "Save Failed", message: String(e), type: "error" });
    }
  }

  function resetAndHide() {
    extensionView = null;
    view = "launcher";
    currentMode = "launcher";
    isVisible = false;
    invoke("hide_window").catch((e) => console.error("Hide failed:", e));
  }

  function closeExtensionAndReturnToLauncher() {
    extensionView = null;
    view = "launcher";
    currentMode = "launcher";

    void maybeRescanApps(true).finally(() => {
      requestAnimationFrame(() => {
        launcherRef?.refreshSuggestions();
        launcherRef?.focus();
      });
    });
  }

  // ── Permission handlers ──
  async function setDecision(decision: "Allow" | "Deny" | "Ask", caps: Capability[], note?: string) {
    if (!permissionPrompt) throw new Error("No permission prompt active");
    await invoke("set_permission_decision", {
      scriptId: permissionPrompt.scriptId,
      requestedCaps: caps,
      decision,
      note,
    });
  }

  async function handleAllowOnce() {
    if (!permissionPrompt) return;
    permissionBusy = true;
    const caps = permissionPrompt.requestedCaps.length
      ? permissionPrompt.requestedCaps
      : permissionPrompt.missingCaps;
    try {
      await setDecision("Allow", caps, "allow-once");
      await setDecision("Ask", caps, "reset-after-once");
      permissionPrompt = null;
    } catch (e) { console.error("Allow once failed", e); }
    finally { permissionBusy = false; }
  }

  async function handleAllowAlways() {
    if (!permissionPrompt) return;
    permissionBusy = true;
    const caps = permissionPrompt.requestedCaps.length
      ? permissionPrompt.requestedCaps
      : permissionPrompt.missingCaps;
    try {
      await setDecision("Allow", caps, "persist");
      permissionPrompt = null;
    } catch (e) { console.error("Allow always failed", e); }
    finally { permissionBusy = false; }
  }

  async function handleDeny() {
    if (!permissionPrompt) return;
    permissionBusy = true;
    const caps = permissionPrompt.requestedCaps.length
      ? permissionPrompt.requestedCaps
      : permissionPrompt.missingCaps;
    try { await setDecision("Deny", caps, "user-deny"); }
    catch (e) { console.error("Deny failed", e); }
    finally { permissionPrompt = null; permissionBusy = false; }
  }

  onDestroy(() => {
    for (const u of unlisteners) { try { u(); } catch {} }
  });

  $effect(() => {
    if (isMiniPlayer) return;
    const current = view;
    void invoke("set_ui_last_view", { view: current }).catch((e) => {
      console.error("Failed to persist active view", e);
    });
  });

  $effect(() => {
    if (isMiniPlayer) return;
    const current = view;
    if (current === viewHistoryCursor) return;

    if (suppressViewHistoryPush) {
      suppressViewHistoryPush = false;
    } else {
      const nextHistory = [...viewHistory, viewHistoryCursor];
      viewHistory = nextHistory.slice(-24);
    }
    viewHistoryCursor = current;
  });

  onMount(async () => {
    isMiniPlayer =
      getCurrentWebviewWindow().label === "spotify-mini" ||
      window.location.search.includes("view=mini-player");

    try {
      unlisteners.push(
        await listen("open_clipboard", async () => {
          currentMode = "clipboard";
          try { clipboardHistory = await invoke("get_clipboard_history"); }
          catch (e) {
            console.error("Failed to fetch history", e);
            notify({ title: "Clipboard Error", message: String(e), type: "error" });
          }
          await invoke("show_window");
        }),
      );
    } catch (e) {
      console.error("Failed to bind clipboard listener", e);
      notify({ title: "Listener Error", message: String(e), type: "error" });
    }

    try { availableThemes = await invoke<ThemeMeta[]>("get_installed_themes"); }
    catch (e) {
      console.error("Failed to load themes:", e);
      notify({ title: "Theme Load Failed", message: String(e), type: "error" });
    }

    try {
      const uiState = await invoke<UiStateSnapshot>("get_ui_state");
      const restoredView = asViewId(uiState?.last_view);
      if (restoredView) {
        view = restoredView;
      }
    } catch (e) {
      console.error("Failed to load UI state", e);
    }

    try {
      const config = await invoke<VantaConfig>("get_config");
      vantaConfig = config;
      applyThemeFromConfig(config);
    } catch (e) {
      console.error("Failed to load config:", e);
      notify({ title: "Config Load Failed", message: String(e), type: "error" });
    }

    if (isMiniPlayer) return;

    window.addEventListener("keydown", handleGlobalWayfindingKeydown, true);
    unlisteners.push(() => window.removeEventListener("keydown", handleGlobalWayfindingKeydown, true));

    try { availableExtensions = await invoke<ExtensionEntry[]>("get_extensions"); }
    catch (e) {
      console.error("Failed to load extensions:", e);
      notify({ title: "Extensions Load Failed", message: String(e), type: "error" });
    }
    try { availableMacros = await invoke<WorkflowMacro[]>("get_workflows"); }
    catch (e) {
      console.error("Failed to load macros:", e);
      notify({ title: "Workflow Load Failed", message: String(e), type: "error" });
    }


    unlisteners.push(
      await listen<VantaConfig>("config-updated", (event) => {
        vantaConfig = event.payload;
        applyThemeFromConfig(event.payload);
        if (currentMode === "launcher") launcherRef?.refreshSuggestions();
      }),
    );

    unlisteners.push(
      await listen<BlurStatus>("blur-status", (event) => { blurMode = event.payload.mode; }),
    );

    unlisteners.push(
      await listen<ExtensionEntry[]>("extensions-changed", (event) => {
        availableExtensions = event.payload;
        if (currentMode === "launcher") launcherRef?.refreshSuggestions();
      }),
    );

    unlisteners.push(
      await listen("apps-changed", () => {
        if (currentMode === "launcher") launcherRef?.refreshSuggestions();
      }),
    );

    unlisteners.push(await listen<WorkflowMacro[]>("macros-changed", (event) => { availableMacros = event.payload; }));


    const appWindow = getCurrentWebviewWindow();
    unlisteners.push(await appWindow.onFocusChanged(({ payload: focused }) => {}));

    unlisteners.push(
      await listen("tauri://show", () => {
        isVisible = true;
        maybeRescanApps();
        requestAnimationFrame(() => {
          if (currentMode === "launcher") {
            launcherRef?.focus();
            launcherRef?.refreshSuggestions();
          }
        });
      }),
    );

    launcherRef?.refreshSuggestions();
    maybeRescanApps(true);
  });
</script>

{#if isMiniPlayer}
  <SpotifyMiniPlayer />
{:else}
  <div
    class="vanta-root vanta-glass v2-shell"
    class:css-blur={blurMode === "fallback"}
    role="application"
  >
    <div class="sr-only" aria-live="polite" aria-atomic="true">{viewAnnouncement}</div>

    {#if view !== "launcher"}
      <div class="global-wayfinding" role="navigation" aria-label="Global wayfinding">
        <div class="global-wayfinding-breadcrumbs">
          {#each breadcrumbTrail as crumb, index}
            <span>{viewLabels[crumb]}</span>
            {#if index < breadcrumbTrail.length - 1}
              <span class="global-wayfinding-separator">/</span>
            {/if}
          {/each}
        </div>
        <div class="global-wayfinding-actions">
          <button class="global-wayfinding-btn" disabled={!canGoBack} onclick={goBackInNavigation}>
            Back
          </button>
          <button class="global-wayfinding-btn" onclick={goHomeInNavigation}>Search</button>
          {#if wayfindingNextAction}
            <button class="global-wayfinding-btn" onclick={() => (view = wayfindingNextAction.target)}>
              {wayfindingNextAction.label}
            </button>
          {/if}
        </div>
        <div class="global-wayfinding-hints">
          <span>Alt+Left</span>
          <span>Alt+Home</span>
          {#if lastReturnView && lastReturnView !== view}
            <span>Return: {viewLabels[lastReturnView]}</span>
          {/if}
          {#if wayfindingNextStep}
            <span>Next: {wayfindingNextStep}</span>
          {/if}
        </div>
      </div>
    {/if}

    {#if view === "store"}
      <div in:fade={{ duration: fadeDuration }} class="shell-pane shell-pane-relative">
        <StoreView onClose={() => (view = "launcher")} onToast={notify} />
      </div>
    {:else if view === "featureHub"}
      <div in:fade={{ duration: fadeDuration }} class="shell-pane shell-pane-relative">
        <FeatureHubWindow
          onClose={() => { view = "launcher"; launcherRef?.refreshSuggestions(); }}
          onOpenCommunity={() => (view = "communityHub")}
          onOpenTheme={() => (view = "themeHub")}
          onOpenExtensions={() => (view = "extensionsHub")}
          onOpenStore={() => (view = "store")}
          onOpenSettings={() => { settingsStartSection = null; view = "settings"; }}
        />
      </div>
    {:else if view === "communityHub" && vantaConfig}
      <div in:fade={{ duration: fadeDuration }} class="shell-pane shell-pane-relative">
        <CommunityHubWindow
          communityFeedOptIn={vantaConfig.general.community_feed_opt_in ?? false}
          onToggleFeedOptIn={async (next) => {
            if (!vantaConfig) return;
            vantaConfig.general.community_feed_opt_in = next;
            try { await invoke("save_config", { newConfig: vantaConfig }); }
            catch (e) {
              console.error("Failed to save community feed opt-in", e);
              notify({ title: "Save Failed", message: String(e), type: "error" });
            }
          }}
          onClose={() => (view = "featureHub")}
        />
      </div>
    {:else if view === "themeHub" && vantaConfig}
      <div in:fade={{ duration: fadeDuration }} class="shell-pane shell-pane-relative">
        <ThemeStudioWindow bind:config={vantaConfig} {availableThemes} onClose={() => (view = "featureHub")} />
      </div>
    {:else if view === "extensionsHub"}
      <div in:fade={{ duration: fadeDuration }} class="shell-pane shell-pane-relative">
        <ExtensionsHubWindow onClose={() => (view = "featureHub")} onOpenStore={() => (view = "store")} />
      </div>
    {:else if view === "settings" && vantaConfig}
      <div in:fade={{ duration: fadeDuration }} class="shell-pane shell-pane-relative">
        <SettingsView
          bind:config={vantaConfig}
          {availableThemes}
          initialSection={settingsStartSection}
          onOpenStore={() => { settingsStartSection = null; view = "store"; }}
          onClose={() => { settingsStartSection = null; view = "launcher"; launcherRef?.refreshSuggestions(); }}
        />
      </div>
    {:else if currentMode === "clipboard"}
      <div in:fade={{ duration: fadeDuration }} class="shell-pane">
        <ClipboardView onEscape={() => resetAndHide()} />
      </div>
    {:else if extensionView}
      <div in:fade={{ duration: fadeDuration }} class="shell-pane">
        <ExtensionHost
          extId={extensionView.extId}
          commandName={extensionView.command}
          extPath={extensionView.extPath}
          onClose={closeExtensionAndReturnToLauncher}
          onToast={(opts) => notify(opts)}
        />
      </div>
    {:else}
      <div in:fade={{ duration: fadeDuration }} class="shell-pane">
        <LauncherView
          bind:this={launcherRef}
          bind:config={vantaConfig}
          {availableExtensions}
          {availableMacros}
          {availableThemes}
          {blurMode}
          bind:permissionPrompt
          {permissionBusy}
          bind:clipboardHistory
          bind:currentMode
          bind:view
          bind:settingsStartSection
          bind:extensionView
          onToast={notify}
          onResetAndHide={resetAndHide}
          onSaveConfigUpdate={saveConfigUpdate}
        />
      </div>
    {/if}

    <ToastContainer />

    {#if permissionPrompt}
      <PermissionModal
        scriptId={permissionPrompt.scriptId}
        missingCaps={permissionPrompt.missingCaps}
        onAllowOnce={handleAllowOnce}
        onAllowAlways={handleAllowAlways}
        onDeny={handleDeny}
        busy={permissionBusy}
      />
    {/if}
  </div>
{/if}

<style>
  .shell-pane {
    height: 100%;
    width: 100%;
  }

  .shell-pane-relative {
    position: relative;
  }

  .global-wayfinding {
    position: absolute;
    right: 12px;
    bottom: 12px;
    z-index: 60;
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: min(540px, calc(100% - 24px));
    max-width: calc(100% - 24px);
    padding: 10px 12px;
    border-radius: 12px;
    border: 1px solid var(--vanta-border, rgba(255, 255, 255, 0.12));
    background: color-mix(in srgb, var(--vanta-surface, #111111) 82%, black 18%);
    box-shadow: 0 10px 22px rgba(0, 0, 0, 0.34);
    backdrop-filter: blur(12px);
  }

  .global-wayfinding-breadcrumbs {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
    font-size: 0.78rem;
    color: var(--vanta-text-secondary, #b6becf);
  }

  .global-wayfinding-separator {
    opacity: 0.7;
  }

  .global-wayfinding-actions {
    display: flex;
    gap: 8px;
  }

  .global-wayfinding-btn {
    border: 1px solid var(--vanta-border, rgba(255, 255, 255, 0.12));
    background: color-mix(in srgb, var(--vanta-bg, #0b0b0c) 68%, white 32%);
    color: var(--vanta-text-primary, #f4f7ff);
    border-radius: 8px;
    font-size: 0.74rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    padding: 5px 10px;
    cursor: pointer;
  }

  .global-wayfinding-btn:disabled {
    opacity: 0.48;
    cursor: default;
  }

  .global-wayfinding-hints {
    display: flex;
    flex-wrap: wrap;
    gap: 7px 12px;
    font-size: 0.72rem;
    color: var(--vanta-text-secondary, #a8b0c0);
  }

  @media (max-width: 720px) {
    .global-wayfinding {
      right: 8px;
      left: 8px;
      bottom: 8px;
      min-width: 0;
      max-width: none;
    }
  }
</style>

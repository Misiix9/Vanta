<script lang="ts">
  let {
    onClose,
    onOpenCommunity,
    onOpenTheme,
    onOpenExtensions,
    onOpenStore,
    onOpenSettings,
  }: {
    onClose: () => void;
    onOpenCommunity: () => void;
    onOpenTheme: () => void;
    onOpenExtensions: () => void;
    onOpenStore: () => void;
    onOpenSettings: () => void;
  } = $props();
</script>

<div class="v2-shell v2-fill hub-window-root">
  <div class="v2-header hub-window-header">
    <div class="window-header-main">
      <span class="window-breadcrumb">Workspace</span>
      <h2>Feature Hub</h2>
    </div>
    <div class="window-actions">
      <button class="link-btn btn-ghost" onclick={onClose} aria-label="Back to launcher">Back</button>
      <button class="link-btn btn-secondary" onclick={onOpenSettings}>Settings</button>
    </div>
  </div>

  <div class="v2-scroll window-scroll">
    <p class="hub-window-subtitle">Everything important is one click away.</p>

    <div class="hub-grid">
      <button class="hub-card v2-card" onclick={onOpenCommunity}>
        <div class="hub-card-icon"><i class="fa-solid fa-users"></i></div>
        <div class="hub-card-title">Community Sharing</div>
        <div class="hub-card-copy">Feedback, voting, snippets, and popular workflows.</div>
      </button>

      <button class="hub-card v2-card" onclick={onOpenTheme}>
        <div class="hub-card-icon"><i class="fa-solid fa-palette"></i></div>
        <div class="hub-card-title">Theme And Profile Studio</div>
        <div class="hub-card-copy">Theme presets, import/export, and profile visuals.</div>
      </button>

      <button class="hub-card v2-card" onclick={onOpenExtensions}>
        <div class="hub-card-icon"><i class="fa-solid fa-wand-magic-sparkles"></i></div>
        <div class="hub-card-title">Extension Creator</div>
        <div class="hub-card-copy">Generate a starter extension template instantly.</div>
      </button>

      <button class="hub-card v2-card" onclick={onOpenStore}>
        <div class="hub-card-icon"><i class="fa-solid fa-store"></i></div>
        <div class="hub-card-title">Extension Store</div>
        <div class="hub-card-copy">Browse and install extensions from Vanta Store.</div>
      </button>
    </div>
  </div>
</div>

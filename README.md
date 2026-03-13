<div align="center">
  <img src="https://raw.githubusercontent.com/Misiix9/vanta/main/src-tauri/icons/128x128.png" width="120" alt="Vanta Logo" />

  # Vanta

  **A hyper-fast, extensible application launcher and command palette for Wayland. (Spotlight alternative)**

  [![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
  [![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org/)
  [![Platform](https://img.shields.io/badge/Platform-Linux%20(Wayland)-green.svg)](https://tauri.app/)
  [![Releases](https://img.shields.io/github/v/release/Misiix9/vanta?label=Latest%20Release)](https://github.com/Misiix9/vanta/releases)
</div>

<br />

## Why Vanta?

Traditional launchers are often slow or clunky. **Vanta** is a fast, Wayland-native command palette built with Rust and Svelte. It starts instantly, stays out of your way, and is fully themeable with plain CSS. With the Extension SDK v2 engine, anyone can build custom commands and full UI screens using TypeScript and Svelte.

---

## Installation

### Arch Linux (AUR)
```bash
yay -S vanta-bin
```

### Ubuntu / Debian
Download the latest `.deb` from [Releases](https://github.com/Misiix9/vanta/releases).
```bash
sudo dpkg -i vanta_5.5.0_amd64.deb
```

### Fedora / OpenSUSE
Download the latest `.rpm` from [Releases](https://github.com/Misiix9/vanta/releases).
```bash
sudo rpm -i vanta-5.5.0-1.x86_64.rpm
```

### Latest Minor (v5.8.0)
- Added in-app toast notifications with auto-dismiss and severity styling (info/success/error).
- Replaced key silent launcher/store failures with user-visible feedback toasts.
- Added store-side update detection for installed extensions using semver comparisons against the registry.
- Added extension update action in Vanta Store (`Update to vX`) when newer versions are available.
- Added extension rollback support with backup snapshots and one-click rollback from Vanta Store.
- Added extension dependency contract support (`requires`) with install-time dependency/version validation.
- Added notification center panel from status bar with recent event history.

### Previous Minor (v5.7.0)
- Added Phase 38 notification and feedback architecture baseline across launcher/store/extension-host flows.

### Previous Minor (v5.6.0)
- Quick Look preview panel (Space key) — shows file metadata, image preview, text content (up to 4 KB), directory listing; non-file results show source info and exec details.
- Right-click context menu on any result row — lists primary action plus all available actions (reveal, copy path, open-with, etc.).
- Image file thumbnails displayed inline in result rows for common image formats (png, jpg, gif, webp, svg, etc.).
- Contextual empty states — filter-specific hints, "no results" messaging, and filter hint chips (`type:app`, `type:file`, `in:clipboard`) on the default empty screen.
- Fuzzy score debug overlay (Ctrl+D) — shows numeric match scores as inline badges on each result for ranking inspection.
- Drag-and-drop from result items — file paths, clipboard text, or exec strings transferred as `text/plain`.

### Previous Minor (v5.5.0)
- Added search source filters: `type:app`, `type:file`, `type:window`, `in:clipboard`, `ext:<id>` narrow results to a specific source; non-matching parallel tasks are skipped entirely for efficiency.
- Added query history with up-arrow recall in empty search input; recent queries (up to 50) persisted in history file with deduplication.
- Added typo tolerance / "Did you mean?" — when results are sparse, Levenshtein edit-distance suggestions from app names surface as clickable fill-query results.
- Contextual search placeholder ("Search apps, files, scripts…") replaces empty placeholder.
- Added searching spinner animation in status bar while queries are in-flight.
- Enhanced `aria-live` result count announcements with explicit `aria-label` for screen readers.
- Made action chips clickable — mouse clicks on action chips now execute the corresponding action (reveal, copy path, open-with, etc.).

### Previous Minor (v5.4.0)
- Parallelized search pipeline using `tokio::join!` / `spawn_blocking` — apps, windows, files, clipboard, and extensions now query concurrently instead of sequentially.
- Added search cancellation via atomic generation counter — new keystrokes abort stale in-flight queries.
- Added result streaming: `search-partial` Tauri events emit partial results as each source completes; frontend merges progressively.
- Replaced raw `HashMap<String, u32>` history counters with time-weighted frecency scoring (`FrecencyEntry` with exponential decay, 12-hour half-life, 10 most recent timestamps).
- Extracted 30+ hardcoded magic scoring numbers into `ranking_config.rs` with documented constants (calculator, windows, clipboard, extensions, profiles, intents, query relevance, source intent, app entity bonuses).
- Added negative scoring: short fuzzy matches penalized, below-threshold results suppressed.
- Backward-compatible history migration: old `vanta_history.json` format auto-migrates to frecency entries on first load.

### Previous Minor (v5.3.0)
- Added `:focus-visible` rings on all interactive elements (buttons, inputs, accordion headers, links) with `!important` to override base resets.
- Added `role="dialog"`, `aria-modal="true"`, and focus trapping on all modal surfaces (PermissionModal, ActionConfirmModal, SettingsView, StoreView).
- Added `aria-live="polite"` regions for result count changes (StatusBar) and view transitions (+page.svelte announcer).
- Replaced global scrollbar kill (`scrollbar-width: none !important`) with accessible thin scrollbar preserving scroll-position feedback.
- Added keyboard shortcut help modal (Ctrl+?) listing all navigation shortcuts with focus trapping.
- Fixed WCAG AA contrast violations: `--text-tertiary` bumped from `#6f7188` to `#8b8da6` (5:1 ratio); `.icon-fallback` opacity raised from 0.5 to 0.72.
- Added skip-navigation link from search input to results region.
- Added `aria-label` to search input for screen reader identification.

### Previous Minor (v5.2.0)
- Extracted `ranking.ts` module with all scoring, command routing, and utility functions (~280 lines of pure logic).
- Split `+page.svelte` (1751 → 343 lines) into a thin view router plus a dedicated `LauncherView.svelte` component (461 lines).
- Split `SettingsView.svelte` (1465 → 226 lines) into a thin orchestrator plus 7 focused sub-components: ThemeSettings, CommunitySettings, FeatureHubSettings, AccessibilitySettings, FileSearchSettings, DiagnosticsSettings.
- Added `LoadingSkeleton.svelte` shimmer component for async loading states.
- Added `ErrorRecovery.svelte` for actionable error recovery UI with retry support.

### Previous Minor (v5.1.0)
- Rewrote `errors.rs` with `thiserror` derive macros — 15 domain-specific error variants (Io, Json, Config, Scanner, Matcher, Launcher, Window, Script, Extension, Store, Workflow, Clipboard, Permission, Community, Theme).
- Added structured error codes (1000–9999) and kind tags; frontend receives `{ code, kind, message }` via Tauri IPC.
- Replaced all `Result<T, String>` with `Result<T, VantaError>` across 109 functions in 10+ modules.
- Added `From<String>` and `From<&str>` bridges for gradual migration; zero production `unwrap()`/`expect()`.

### Previous Patch (v5.0.1)
- Replaced per-call `Regex::new().unwrap()` in workflow token rendering with a static `LazyLock<Regex>` (crash prevention).
- Added length and null-byte validation on `$TERMINAL` env before `shell_words` parsing (DoS mitigation).
- Wrapped root layout in `<svelte:boundary>` error boundary — uncaught JS errors now show a recovery UI instead of killing the window.
- Bundled Font Awesome locally via `@fortawesome/fontawesome-free` — removed CDN dependency for offline resilience.

### Previous Major (v5.0.0)
- Added Phase 30 Theme Engine v3 with 5-layer token inheritance (Context → Core → Semantic → Component → Compatibility).
- Added theme token contract enforcing 15 required and 25 recommended tokens with diagnostics.
- Added dual-side theme validation in both TypeScript and Rust with error/warning reporting.
- Improved legacy alias safety by converting hardcoded values to var() references across both built-in themes.

### Previous Minor (v4.9.0)
- Added Phase 29 iconography system with size tokens from --icon-xs through --icon-4xl.

### Previous Minor (v4.8.0)
- Added Phase 28 typography token system with heading, body, and micro type scales.

### Previous Minor (v4.7.0)
- Added Phase 27 motion system primitives with reduced-motion-safe fallbacks across major views.
- Added consistent micro-interaction choreography for hover, focus, selection, and action feedback.
- Added full Aurora Slate redesign for universal theme surfaces with richer depth and hierarchy.
- Improved interaction performance with non-blocking transform/opacity-first motion behavior.

### Previous Minor (v4.6.0)
- Added Phase 26 Dedicated Window UX Suite v2 with shared shell, header, action, and scroll primitives.
- Added consistent breadcrumb and action grammar across dedicated hub windows and the extension runtime host.
- Improved responsive dedicated-window behavior with stacked action bars and tighter mobile-safe panel spacing.
- Improved maintainability by replacing remaining inline dedicated-window block layout styles.

### Previous Minor (v4.5.0)
- Added Phase 25 Store and extensions surface redesign with shared visual primitives.
- Improved trust and risk metadata scanability in store cards with clearer grouping.
- Improved install and uninstall action feedback consistency across extension surfaces.
- Improved settings extensions section with extension-surface parity and direct store access.

### Previous Minor (v4.4.0)
- Added Phase 24 settings IA refresh with sticky settings header and in-settings section search jump.
- Added canonical settings section alias handling for more reliable deep-link navigation.
- Improved settings panel consistency by replacing inline layout styles with reusable utility classes.
- Added shared primitives for hint lists and hub-window shell layout.

### Previous Minor (v4.3.0)
- Added Phase 23 launcher layout recomposition with balanced search, results, and status regions.
- Added density-aware spacing behavior across launcher surfaces using shared spacing tokens.
- Improved result row affordances with clearer hover/selected distinction and cleaner action hint rhythm.
- Improved grouped-mode navigation feedback with selected-state section headers.

### Previous Minor (v4.2.0)
- Added Phase 22 UI primitive foundations with reusable shell, panel, card, and stack patterns.
- Added expanded semantic token mapping with runtime fallbacks for elevated surfaces and status roles.
- Improved launcher and settings visual consistency with density-aware spacing and shared layout primitives.
- Improved result and status surfaces with cleaner hierarchy and tokenized spacing.

### Previous Minor (v4.1.0)
- Added Phase 21 Smartest Search And Result Intelligence with relevance-first ordering for typed queries.
- Added app-aware entity resolution for multi-step intents (for example: `open zen then open foot`).
- Added query-aware ranking bonuses to boost exact and prefix matches.
- Improved case-insensitive fuzzy matching and launch-history-driven ranking.

### Previous Major (v4.0.0)
- Added Phase 20 Adaptive Intent Engine for local-first intent parsing.
- Added multi-step workflow suggestions from natural queries (for example: `open settings then open store`).
- Added explicit confirmation before running inferred multi-step workflows.
- Added typed `intent_workflow` command contract integration in launcher command handling.

### Previous Minor (v3.6.0)
- Added Phase 19 Community panel with in-app feedback submission and roadmap voting hooks.
- Added trusted shareable snippet export/import for workflows, profiles, and theme presets.
- Added opt-in Popular Workflows discovery feed with install actions into local macros.
- Added safety guardrails to block risky unverified workflow snippet imports.

### Previous Major (v3.0.0)
- Added typed result/action contract (`search_v3`, `get_suggestions_v3`) with legacy `exec` fallback.
- Added versioned extension manifest schema (`schema_version`) and automatic local migration.
- Added versioned config/workflow schema fields and migration tooling (`run_contract_migration`).
- Backward compatibility policy: legacy `exec` remains available during the v3 transition window.

---

## Features

- **Fast fuzzy search** powered by Rust + `nucleo-matcher`.
- **Extension SDK v2**: Build custom commands and full UI screens with TypeScript/Svelte.
- **Vanta Store** (v2.1): Browse and install extensions from the built-in store. Search "store" or "install" in the launcher.
- **10 Default Extensions**: Weather, Smart Calculator, Color Picker, Process Manager, Network Test, Timer, System Info, Spotify, Clipboard Tools, and Password Generator.
- **Auto-refresh** (v2.2): Settings changes, new app installations, and new extensions take effect immediately without restarting.
- **Commands section**: Sleep, Lock, Shutdown, Restart, Log Out, and Go to BIOS available out of the box.
- **Clipboard-first**: `--clipboard` launch and `Super+V` open the clipboard view.
- **File search with filters**: Include/exclude globs, extension allowlist, and type filters.
- **Window switcher**: Grouped by app/class, ordered by recency, with focus and close actions.
- **Math & calculator**: Type `2^10 + 5` to copy the result.
- **Fully themeable**: Everything is CSS in `~/.config/vanta/themes/`.
- **Workflow macros**: Chain system commands and extension actions into automated sequences.

---

## Quick Start

```bash
git clone https://github.com/Misiix9/vanta.git
cd vanta
npm install
cargo tauri dev
```

**Flags:**
- `--hidden` or `VANTA_HIDDEN=1`: start minimized/hidden.
- `--clipboard` or `-c`: open directly to clipboard mode on launch.
- **Hotkeys:** `Alt+Space` (toggle), `Super+V` (clipboard).

---

## Extension SDK

Vanta Extension SDK v2 replaces the old JSON script system with a full Raycast-style extension engine. Extensions live in `~/.config/vanta/extensions/` and can declare multiple commands with custom UI.

SDK v2 authoring toolkit (in this repo):
- `vanta-extensions/templates/view-template/` starter extension
- `npm run extensions:validate` manifest linting
- `npm run extensions:harness` extension bundle/command harness checks
- `vanta-extensions/compatibility-matrix.json` compatibility + deprecation lifecycle contract
- `vanta-extensions/ratings.json` shared rating snapshot consumed by Vanta Store

### Extension Structure

```
~/.config/vanta/extensions/my-extension/
  manifest.json       # Metadata, commands, permissions
  dist/
    index.js          # Compiled IIFE bundle
    style.css         # Optional scoped styles
```

### Manifest Format

```json
{
  "name": "my-extension",
  "title": "My Extension",
  "schema_version": 1,
  "version": "1.0.0",
  "description": "What this extension does",
  "author": "your-name",
  "icon": "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2'><polygon points='12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2'/></svg>",
  "permissions": ["Network", "Shell"],
  "commands": [
    {
      "name": "quick-action",
      "title": "Quick Action",
      "subtitle": "Does something fast",
      "mode": "no-view",
      "icon": "fa-solid fa-bolt"
    },
    {
      "name": "browse-items",
      "title": "Browse Items",
      "subtitle": "Opens a searchable list",
      "mode": "view",
      "icon": "fa-solid fa-list"
    }
  ]
}
```

### Command Modes

- **`no-view`**: Runs a handler function (e.g., toggle playback, copy text), shows a toast, and optionally closes the window.
- **`view`**: Mounts a Svelte component inside the extension host for full custom screen rendering.

### Writing an Extension

Extensions compile to IIFE bundles that register via `window.__vanta_host`:

```javascript
(function(vanta) {
  const sdk = window.__vanta_sdk;

  vanta.registerExtension('my-extension', {
    commands: {
      'quick-action': {
        handler: async (api) => {
          api.toast({ title: 'Done!', type: 'success' });
          await api.closeMainWindow();
        }
      },
      'browse-items': {
        component: sdk.List  // Use a pre-built SDK component
      }
    }
  });
})(window.__vanta_host);
```

### SDK Components

Extensions can use pre-built Svelte components via `window.__vanta_sdk`:

| Component | Description |
| :--- | :--- |
| `sdk.List` | Searchable, keyboard-navigable list with sections |
| `sdk.Form` | Form with text, password, dropdown, checkbox, and date fields |
| `sdk.Grid` | Visual grid layout with images/icons |
| `sdk.Detail` | Detail view with content and metadata sidebar |
| `sdk.ActionPanel` | Action panel overlay with keyboard shortcuts |

### VantaAPI

Every command handler and view component receives a `VantaAPI` object:

```typescript
api.navigation.push(component, props)  // Push a new view
api.navigation.pop()                    // Go back
api.clipboard.copy(text)                // Copy to clipboard
api.network.fetch(url, { method })      // HTTP request (proxied through Rust)
api.shell.execute(command, args)        // Run a shell command
api.storage.get(key)                    // Read from per-extension storage
api.storage.set(key, value)             // Write to per-extension storage
api.toast({ title, message, type })     // Show a toast
api.closeMainWindow()                   // Hide the launcher
api.environment.extensionName           // Current extension name
```

### Building with Vite

```typescript
// vite.config.ts
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  build: {
    lib: {
      entry: 'src/index.ts',
      formats: ['iife'],
      name: 'MyExtension',
      fileName: () => 'index.js',
    },
    outDir: 'dist',
    rollupOptions: {
      external: ['@vanta/sdk'],
      output: { globals: { '@vanta/sdk': 'window.__vanta_sdk' } },
    },
  },
});
```

### Migration from v1.x Scripts

The v1.x JSON script system (`~/.config/vanta/scripts/`) has been fully removed. To migrate:

1. Create a new extension directory in `~/.config/vanta/extensions/your-script/`
2. Write a `manifest.json` with your command metadata
3. Convert your script logic to a JavaScript handler
4. Build to `dist/index.js` (or write the IIFE directly)

### v3 Contract Compatibility Policy

- Search APIs: prefer `search_v3` and `get_suggestions_v3`.
- Result/action execution: use typed `command.kind` when present, otherwise fall back to legacy `exec`.
- Extension manifests: `schema_version` is required for new manifests; legacy manifests are auto-migrated to current schema on scan.
- Config/workflows: `schema_version` and `workflows.schema_version` are maintained automatically on load or by running `run_contract_migration`.

---

## Settings & Theming

All styling lives in `~/.config/vanta/themes/`. A default theme is seeded on first launch.

Key CSS variables:
- `--vanta-width` / `--vanta-height`: window size
- `--vanta-accent`: accent color
- `--vanta-blur`, `--vanta-radius`, `--vanta-opacity`: glass look

---

## Configuration

Configuration lives at `~/.config/vanta/config.json`, created automatically on first run.

<details>
  <summary><strong>View Default Configuration</strong></summary>

```json
{
  "schema_version": 4,
  "general": {
    "hotkey": "Alt+Space",
    "max_results": 8,
    "launch_on_login": false
  },
  "appearance": {
    "blur_radius": 40,
    "opacity": 0.85,
    "border_radius": 24,
    "theme": "default",
    "colors": {
      "background": "#000000",
      "surface": "#0a0a0a",
      "accent": "#ffffff",
      "accent_glow": "rgba(255,255,255,0.1)",
      "text_primary": "#f5f5f5",
      "text_secondary": "#888888",
      "border": "rgba(255,255,255,0.05)"
    }
  },
  "extensions": {
    "directory": "~/.config/vanta/extensions",
    "dev_mode": false
  },
  "files": {
    "include_hidden": false,
    "max_depth": 3,
    "file_manager": "default",
    "file_editor": "default",
    "open_docs_in_manager": false,
    "include_globs": [],
    "exclude_globs": [],
    "allowed_extensions": [],
    "type_filter": "any",
    "indexed_at": null
  },
  "workflows": {
    "schema_version": 1,
    "macros": []
  }
}
```
</details>

---

## Keybindings

| Key | Action |
| :--- | :--- |
| `Alt + Space` | Toggle window |
| `Super + V` | Open clipboard history |
| `Ctrl + ,` | Open settings |
| `Esc` | Close window / Back (in extensions) |
| `Enter` | Launch / Execute |
| `Arrow Up / Down` | Navigate results |
| `Tab` | Autocomplete / Next result |

---

<div align="center">
  <sub>Made with love by <a href="https://github.com/Misiix9">onxy</a></sub>
</div>

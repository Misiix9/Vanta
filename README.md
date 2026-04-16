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
sudo dpkg -i vanta_5.12.0_amd64.deb
```

### Fedora / OpenSUSE
Download the latest `.rpm` from [Releases](https://github.com/Misiix9/vanta/releases).
```bash
sudo rpm -i vanta-5.12.0-1.x86_64.rpm
```

### Latest Minor (v5.12.0)
- Extracted shared theme/component primitives into `base.css` and kept theme files focused on tokens and overrides.
- Added release-time visual budget checks for inline layout styles and theme CSS size drift.
- Added icon cache bounds and integrity pruning for stale or invalid cached icon paths.
- Lazy-loaded non-critical hub surfaces (`Store`, `Community Hub`, `Theme Studio`, `Extensions Hub`) to reduce startup/bundle waste.
- Expanded file-type icon coverage for Java, C/C++, Python, Go, Kotlin, Ruby, and Office formats.
- Pinned `svelte` to an exact version for build stability across releases.

### Previous Minor (v5.11.0)
- Added calculator upgrades for unit conversion, timezone lookup, number-base parsing, and offline currency conversion.
- Added smart clipboard contextual actions, persistent bookmarks, quick notes, command palette mode, and local analytics surfaces.

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

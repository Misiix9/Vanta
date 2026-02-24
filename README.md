<div align="center">
  <img src="https://raw.githubusercontent.com/Misiix9/vanta/main/src-tauri/icons/128x128.png" width="120" alt="Vanta Logo" />

  # Vanta

  **A hyper-fast, scriptable application launcher and command palette for Wayland. (Spotlight alternative)**

  [![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
  [![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org/)
  [![Platform](https://img.shields.io/badge/Platform-Linux%20(Wayland)-green.svg)](https://tauri.app/)
  [![Releases](https://img.shields.io/github/v/release/Misiix9/vanta?label=Latest%20Release)](https://github.com/Misiix9/vanta/releases)
</div>

<br />

## Why Vanta?

Traditional launchers are often slow or clunky. **Vanta** is a fast, Wayland-native command palette built with Rust and Svelte. It starts instantly, stays out of your way, and is fully themeable with plain CSS. If your script can output JSON, Vanta can run it.

---

## Installation

### Arch Linux (AUR)
```bash
yay -S vanta-bin
```

### Ubuntu / Debian
Download the latest `.deb` from [Releases](https://github.com/Misiix9/vanta/releases).
```bash
sudo dpkg -i vanta_1.15.0_amd64.deb
```

### Fedora / OpenSUSE
Download the latest `.rpm` from [Releases](https://github.com/Misiix9/vanta/releases).
```bash
sudo rpm -i vanta-1.15.0-1.x86_64.rpm
```

### AppImage (Universal)
```bash
chmod +x Vanta_1.15.0_amd64.AppImage
./Vanta_1.15.0_amd64.AppImage
```

---

## Features

- **Fast fuzzy search** powered by Rust + `nucleo-matcher`.
- **Commands section**: Sleep, Lock, Shutdown, Restart, Log Out, and Go to BIOS (firmware) available out of the box.
- **Scripts section**: Installed scripts are listed together and refresh automatically after install.
- **Clipboard-first**: `--clipboard` launch and `Super+V` open the clipboard view; history loads instantly and refocuses input.
- **File search with filters**: Include/exclude globs, extension allowlist, and type filters; quick manual index rebuild.
- **Window switcher v2**: Grouped by app/class, ordered by recency, with focus and close actions.
- **Math & calculator**: Type `2^10 + 5` to copy the result.
- **Script Store**: `install <GitHub-URL>` or `install /path/to/script.sh` to add plugins.
- **Fully themeable**: Everything is CSS in `~/.config/vanta/themes/`; adjust colors, blur, radius, and layout.
- **Resilient startup**: Honors config window size, clamps bounds, recenters before show, and supports hidden startup.

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

**Fresh installs:** Vanta seeds `~/.config/vanta/config.json` and `~/.config/vanta/themes/default.css` if missing.

**Built-in commands:** Sleep, Lock, Shutdown, Restart, Log Out, Go to BIOS — available in the Commands section.

**Scripts:** Installed scripts appear in the Scripts section and auto-refresh after install.

---

## Settings & Theming

All styling lives in `~/.config/vanta/themes/`. A default theme is seeded on first launch; duplicate and edit freely.

Key CSS variables:
- `--vanta-width` / `--vanta-height`: window size
- `--vanta-accent`: accent color
- `--vanta-blur`, `--vanta-radius`, `--vanta-opacity`: glass look

Set a theme name via `/* Theme Name: ... */` to surface it in Settings.

Install a theme from a URL:
```
install https://example.com/my-theme.css
```

---

## Scripting

Vanta scans `~/.config/vanta/scripts/` for executable files. A script outputs JSON and Vanta renders it.

**Example: `~/.config/vanta/scripts/hello.sh`**
```bash
#!/bin/bash
# vanta:name=Hello World
# vanta:description=A simple test script
# vanta:icon=star

echo '{
  "items": [
    {
      "title": "Hello from Bash!",
      "subtitle": "Click to copy",
      "action": { "type": "copy", "value": "Hello Vanta!" }
    }
  ]
}'
```
Type `hello` in Vanta and this result appears instantly. Actions can be `copy`, `open`, or `run`.

To install a script from GitHub:
```
install https://github.com/user/vanta-scripts
```

### Script validation

Run `vanta doctor` to lint installed scripts (metadata, executable bit, JSON schema). You can also type “vanta doctor” in the launcher to open a terminal and run it. Enable strict JSON validation at runtime in **Settings → Scripts**.

---

## Configuration

Configuration lives at `~/.config/vanta/config.json`, created automatically on first run.

Window search cap can be tuned with `search.windows_max_results` (0 uses half of `general.max_results`).

<details>
  <summary><strong>View Default Configuration</strong></summary>

```json
{
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
  "scripts": {
    "directory": "~/.config/vanta/scripts",
    "timeout_ms": 5000,
    "strict_json": false
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
| `Esc` | Close window |
| `Enter` | Launch / Execute |
| `Arrow Up / Down` | Navigate results |
| `Tab` | Autocomplete / Next result |

---

<div align="center">
  <sub>Made with ♥️ by <a href="https://github.com/Misiix9">onxy</a></sub>
</div>

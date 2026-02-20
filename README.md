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

Traditional application launchers are often bloated, slow, or ugly. Vanta is built differently. It leverages **Rust** for instant startup times and uses a transparent, borderless UI designed specifically for Wayland compositors like Hyprland and Sway.

Vanta is also deeply scriptable — if your script can output JSON, Vanta can run it. Calculate math, control Spotify, fetch weather, or manage Docker containers without ever leaving your keyboard.

Since v1.8.0, Vanta's visual style is **entirely driven by CSS files** you control. There is no hardcoded theme. Every color, radius, font, and layout dimension is overridable.

---

## Installation

### Arch Linux (AUR)
```bash
yay -S vanta-bin
```

### Ubuntu / Debian
Download the latest `.deb` from [Releases](https://github.com/Misiix9/vanta/releases).
```bash
sudo dpkg -i vanta_1.8.0_amd64.deb
```

### Fedora / OpenSUSE
Download the latest `.rpm` from [Releases](https://github.com/Misiix9/vanta/releases).
```bash
sudo rpm -i vanta-1.8.0-1.x86_64.rpm
```

### AppImage (Universal)
```bash
chmod +x Vanta_1.8.0_amd64.AppImage
./Vanta_1.8.0_amd64.AppImage
```

---

## Features

- **Blazing Fast**: Powered by Rust and `nucleo-matcher` for sub-millisecond fuzzy search.
- **Fully Themeable**: All styles live in `~/.config/vanta/themes/`. Drop in any `.css` file, select it in Settings, and the app resizes and reskins instantly — no restart required.
- **Scriptable**: Write plugins in Python, Bash, Node.js, or any language. If it outputs JSON, Vanta can run it.
- **Clipboard History**: Press `Super+V` to open a searchable clipboard history.
- **File Search**: Search and open files from your home directory.
- **Math**: Type an expression like `2^10 + 5` and get the result instantly.
- **Script Store**: Install community scripts with `install <GitHub-URL>` or `install /path/to/script.sh`.
- **Borderless Glass Window**: Fully transparent and undecorated window — the OS frame is replaced entirely by your CSS.

---

## Theming

Vanta's entire visual appearance is controlled by CSS files in `~/.config/vanta/themes/`.

A default theme is seeded to `~/.config/vanta/themes/default.css` on first launch. To create a custom theme, duplicate it and edit freely.

### Theme File Format

```css
/* Theme Name: My Theme */
:root {
  --vanta-width: 680px;   /* Window width */
  --vanta-height: 420px;  /* Window height */

  --vanta-bg: #0a0a0a;
  --vanta-text: #f5f5f5;
  --vanta-text-dim: #888888;
  --vanta-accent: #ffffff;
  --vanta-blur: 40px;
  --vanta-radius: 24px;
  --vanta-opacity: 0.88;
}

/* Add any CSS you want below */
```

The comment `/* Theme Name: ... */` sets the display name in the Settings dropdown. `--vanta-width` and `--vanta-height` control the window size when this theme is active.

To install a theme from a URL:
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

---

## Configuration

Configuration lives at `~/.config/vanta/config.json`, created automatically on first run.

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
    "timeout_ms": 2000
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
  <sub>Built by <a href="https://github.com/Misiix9">Misiix9</a></sub>
</div>

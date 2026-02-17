<div align="center">
  <img src="https://raw.githubusercontent.com/Misiix9/vanta/main/src-tauri/icons/128x128.png" width="120" alt="Vanta Logo" />

  # Vanta

  **A hyper-fast, scriptable command palette for Wayland.**

  [![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
  [![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org/)
  [![Platform](https://img.shields.io/badge/Platform-Linux%20(Wayland)-green.svg)](https://tauri.app/)
  [![Releases](https://img.shields.io/github/v/release/Misiix9/vanta?label=Latest%20Release)](https://github.com/Misiix9/vanta/releases)
</div>

<br />

## Why Vanta?

Traditional application launchers are often bloated, slow, or ugly. **Vanta** is built differently. It leverages **Rust** for instant startup times and uses a modern "Liquid Glass" UI designed specifically for Wayland compositors like Hyprland and Sway.

But speed isn't enough. Vanta is deeply scriptable—if your script can output JSON, Vanta can run it. Calculate math, control Spotify, fetch weather, or manage Docker containers without ever leaving your keyboard.

## Installation

### Arch Linux (AUR)
Install the binary package from the AUR:
```bash
yay -S vanta-bin
```

### Ubuntu / Debian
Download the latest `.deb` from [Releases](https://github.com/Misiix9/vanta/releases).
```bash
sudo dpkg -i vanta_1.0.0_amd64.deb
```

### Fedora / OpenSUSE
Download the latest `.rpm` from [Releases](https://github.com/Misiix9/vanta/releases).
```bash
sudo rpm -i vanta-1.0.0-1.x86_64.rpm
```

### AppImage (Universal)
Works on any Linux distribution. Download, make executable, and run.
```bash
chmod +x Vanta_1.0.0_amd64.AppImage
./Vanta_1.0.0_amd64.AppImage
```

---

## Features

- 🚀 **Blazing Fast**: Powered by Rust and `nucleo-matcher` for sub-millisecond search results.
- 💎 **Liquid Glass UI**: A frosted glass aesthetic that feels native on modern Linux desktops.
- ⚡ **Scriptable**: Write plugins in Python, Bash, Node.js, or Ruby.
- 🐧 **Universal**: Built on Tauri v2, supporting all major Linux distributions.

## Scripting Power

Vanta scans `~/.config/vanta/scripts/` for executable files. A script simply needs to output JSON.

**Example: `~/.config/vanta/scripts/hello.sh`**
```bash
#!/bin/bash
# vanta:name=Hello World
# vanta:description=A simple test script
# vanta:icon=👋

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
Type `hello` in Vanta, and this result appears instantly.

## Configuration

Configuration lives at `~/.config/vanta/config.json`. It's automatically created on first run.

<details>
  <summary><strong>View Default Configuration</strong></summary>

```json
{
  "general": {
    "hotkey": "Alt+Space",
    "max_results": 50,
    "launch_on_login": false
  },
  "appearance": {
    "blur_radius": 20,
    "opacity": 0.85,
    "border_radius": 16,
    "colors": {
      "background": "rgba(20, 20, 30, 0.7)",
      "surface": "rgba(255, 255, 255, 0.05)",
      "accent": "#8b5cf6",
      "accent_glow": "rgba(139, 92, 246, 0.3)",
      "text_primary": "#ffffff",
      "text_secondary": "rgba(255, 255, 255, 0.6)",
      "border": "rgba(255, 255, 255, 0.1)"
    }
  },
  "scripts": {
    "directory": "~/.config/vanta/scripts",
    "timeout_ms": 2000
  }
}
```
</details>

## Keybindings

| Key | Action |
| :--- | :--- |
| **Alt + Space** | Toggle Window |
| **Esc** | Close Window |
| **Enter** | Launch / Execute |
| **↑ / ↓** | Navigate Results |
| **Tab** | Next Result |

---

<div align="center">
  <sub>Built with ❤️ by <a href="https://github.com/Misiix9">onxy</a></sub>
</div>

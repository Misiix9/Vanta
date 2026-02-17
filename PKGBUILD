# Maintainer: onxy <onxy@dev.vanta.app>
pkgname=vanta
pkgver=1.0.0
pkgrel=1
pkgdesc="Liquid Glass command palette for Linux (Tauri v2 + Svelte 5)"
arch=('x86_64')
url="https://github.com/yourusername/vanta"
license=('MIT')
depends=('webkit2gtk-4.1' 'gtk3' 'libappindicator-gtk3' 'fuse2') # Added fuse2 explicitly
provides=('vanta')
conflicts=('vanta')
# No source needed for local binary packaging wrapper
options=('!strip') # Don't strip debug symbols if we want them, or let it strip.

package() {
    # Define paths relative to this PKGBUILD
    local bin_path="$startdir/src-tauri/target/release/vanta"
    local icon_path="$startdir/src-tauri/icons/128x128.png"
    local desktop_path="$startdir/vanta.desktop"

    if [ ! -f "$bin_path" ]; then
        echo "Error: Binary not found at $bin_path"
        echo "Please run 'cargo tauri build' first!"
        return 1
    fi

    # Install binary
    install -Dm755 "$bin_path" "$pkgdir/usr/bin/vanta"

    # Install icon
    install -Dm644 "$icon_path" "$pkgdir/usr/share/icons/hicolor/128x128/apps/vanta.png"

    # Install desktop entry
    install -Dm644 "$desktop_path" "$pkgdir/usr/share/applications/vanta.desktop"
}

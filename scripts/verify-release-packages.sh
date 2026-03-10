#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

mapfile -t debs < <(find src-tauri/target -type f -path '*/bundle/deb/*.deb' | sort)
mapfile -t rpms < <(find src-tauri/target -type f -path '*/bundle/rpm/*.rpm' | sort)

if [[ ${#debs[@]} -eq 0 && ${#rpms[@]} -eq 0 ]]; then
  echo "No release packages found under src-tauri/target/**/bundle." >&2
  exit 1
fi

echo "Verifying release package metadata..."
for pkg in "${debs[@]}"; do
  echo "[deb] $pkg"
  dpkg-deb --info "$pkg" >/dev/null
  dpkg-deb --contents "$pkg" >/dev/null
  echo "  ok"
done

for pkg in "${rpms[@]}"; do
  echo "[rpm] $pkg"
  rpm -qpi "$pkg" >/dev/null
  rpm -qpl "$pkg" >/dev/null
  echo "  ok"
done

echo "Package verification completed successfully."

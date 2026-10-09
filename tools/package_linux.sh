#!/usr/bin/env bash
# Build a Linux tar.gz (and an AppImage if appimagetool is installed).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"; cd "$ROOT"
OUT="dist/linux/Ultima67"
cargo run -q --release -p u67_assetgen
cargo build --release -p u67_game
rm -rf "$OUT"; mkdir -p "$OUT"
cp target/release/ultima67 "$OUT/"
rsync -a --exclude original --exclude maps assets/ "$OUT/assets/"
cp README.md LICENSE commands.txt ASSETS.md "$OUT/"
tar -C dist/linux -czf dist/linux/Ultima67-linux-x64.tar.gz Ultima67
echo "Built dist/linux/Ultima67-linux-x64.tar.gz"
if command -v appimagetool >/dev/null; then
  APPDIR="dist/linux/Ultima67.AppDir"; rm -rf "$APPDIR"; mkdir -p "$APPDIR/usr/bin"
  cp -r "$OUT"/* "$APPDIR/usr/bin/"
  cp assets/icon/icon.png "$APPDIR/ultima67.png"
  printf '[Desktop Entry]\nName=Ultima67\nExec=ultima67\nIcon=ultima67\nType=Application\nCategories=Game;RolePlaying;\n' > "$APPDIR/ultima67.desktop"
  printf '#!/bin/sh\ncd "$(dirname "$0")/usr/bin"\nexec ./ultima67 "$@"\n' > "$APPDIR/AppRun"; chmod +x "$APPDIR/AppRun"
  appimagetool "$APPDIR" dist/linux/Ultima67-x86_64.AppImage
fi

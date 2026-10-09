#!/usr/bin/env bash
# Build a native Apple-Silicon (or Intel) Ultima67.app and a .dmg.
# Usage: tools/bundle_macos.sh [aarch64-apple-darwin|x86_64-apple-darwin]
set -euo pipefail
TARGET="${1:-aarch64-apple-darwin}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/dist/macos"
APP="$OUT/Ultima67.app"
cd "$ROOT"
rustup target add "$TARGET" >/dev/null 2>&1 || true
cargo run -q --release -p u67_assetgen            # make sure generated assets are fresh
cargo build --release -p u67_game --target "$TARGET"
rm -rf "$APP"; mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "target/$TARGET/release/ultima67" "$APP/Contents/MacOS/ultima67"
# the game looks for <exe>/../Resources/assets
rsync -a --exclude original --exclude maps "$ROOT/assets/" "$APP/Contents/Resources/assets/"
# icon: assets/icon/icon.png -> icns
if command -v sips >/dev/null && command -v iconutil >/dev/null; then
  ICONSET="$OUT/Ultima67.iconset"; rm -rf "$ICONSET"; mkdir -p "$ICONSET"
  for s in 16 32 64 128 256; do sips -z $s $s "assets/icon/icon.png" --out "$ICONSET/icon_${s}x${s}.png" >/dev/null; done
  cp "$ICONSET/icon_32x32.png"  "$ICONSET/icon_16x16@2x.png"
  cp "$ICONSET/icon_64x64.png"  "$ICONSET/icon_32x32@2x.png"
  cp "$ICONSET/icon_256x256.png" "$ICONSET/icon_128x128@2x.png"
  iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/Ultima67.icns"
fi
cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleName</key><string>Ultima67</string>
  <key>CFBundleDisplayName</key><string>Ultima67</string>
  <key>CFBundleIdentifier</key><string>com.crowelian.ultima67</string>
  <key>CFBundleVersion</key><string>0.1.0</string>
  <key>CFBundleShortVersionString</key><string>0.1.0</string>
  <key>CFBundleExecutable</key><string>ultima67</string>
  <key>CFBundleIconFile</key><string>Ultima67</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHumanReadableCopyright</key><string>By Crowelian 2026 + Sonnet. GPL-3.0-or-later.</string>
</dict></plist>
PLIST
# ad-hoc sign so Apple Silicon will run it locally (use a Developer ID + notarytool for distribution)
codesign --force --deep -s - "$APP" || echo "codesign skipped"
rm -f "$OUT/Ultima67.dmg"
hdiutil create -volname Ultima67 -srcfolder "$APP" -ov -format UDZO "$OUT/Ultima67.dmg"
echo "Built: $APP  and  $OUT/Ultima67.dmg"
echo "First launch: right-click -> Open (ad-hoc signed). For release: codesign with a Developer ID and run xcrun notarytool."

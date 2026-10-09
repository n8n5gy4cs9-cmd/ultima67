#!/usr/bin/env bash
# Builds dist/macos/Ultima67.app (native for this Mac) and a .dmg.
set -euo pipefail
cd "$(dirname "$0")"
case "$(uname -m)" in
  arm64) exec tools/bundle_macos.sh aarch64-apple-darwin ;;
  *)     exec tools/bundle_macos.sh x86_64-apple-darwin ;;
esac

#!/usr/bin/env bash
# Runs the built app (builds it first if missing). Extra args are passed to the game.
set -euo pipefail
cd "$(dirname "$0")"
APP="dist/macos/Ultima67.app"
[ -d "$APP" ] || ./build.sh
exec "$APP/Contents/MacOS/ultima67" "$@"

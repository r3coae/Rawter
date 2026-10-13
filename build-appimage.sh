#!/usr/bin/env bash
# Builds rawter-x86_64.AppImage from the Rust project in the current directory.
# Usage:  ./build-appimage.sh            (runs cargo build --release first)
#         BIN=path/to/rawter ./build-appimage.sh   (skip cargo, package this binary)
set -euo pipefail
cd "$(dirname "$0")"

APP=rawter
ARCH=x86_64
OUT="${APP}-${ARCH}.AppImage"
APPDIR="build/${APP}.AppDir"

if [ -z "${BIN:-}" ]; then
  cargo build --release
  BIN="target/release/${APP}"
fi
[ -x "$BIN" ] || { echo "binary not found: $BIN" >&2; exit 1; }

rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
install -m755 "$BIN" "$APPDIR/usr/bin/${APP}"
install -m644 packaging/${APP}.desktop "$APPDIR/${APP}.desktop"
install -m644 assets/${APP}.png "$APPDIR/${APP}.png"
ln -sf "${APP}.png" "$APPDIR/.DirIcon"

cat > "$APPDIR/AppRun" << 'RUN'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
exec "$HERE/usr/bin/rawter" "$@"
RUN
chmod +x "$APPDIR/AppRun"

TOOL="build/appimagetool"
if [ ! -x "$TOOL" ]; then
  mkdir -p build
  curl -fL -o "$TOOL" \
    "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-${ARCH}.AppImage"
  chmod +x "$TOOL"
fi

# --appimage-extract-and-run lets this work without FUSE (CI, containers)
ARCH=$ARCH "$TOOL" --appimage-extract-and-run "$APPDIR" "$OUT"
chmod +x "$OUT"
echo "Built $OUT"

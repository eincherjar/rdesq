#!/usr/bin/env bash
set -euo pipefail

APP="rdesq"
ARCH="x86_64"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RELEASE_DIR="$SCRIPT_DIR/../src-tauri/target/release"
APPDIR="$RELEASE_DIR/$APP.AppDir"
BINARY="$RELEASE_DIR/$APP"
VERSION="$(jq -r '.version' "$SCRIPT_DIR/../src-tauri/tauri.conf.json")"

cleanup() { rm -rf "$APPDIR"; }
trap cleanup EXIT

echo "[*] Creating AppImage for $APP v$VERSION"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"

cp "$BINARY" "$APPDIR/usr/bin/$APP"

SHARUN_BIN="$APPDIR/sharun"
if [ ! -f "$SHARUN_BIN" ]; then
  echo "[*] Downloading sharun..."
  wget -qO "$SHARUN_BIN" \
    "https://github.com/pkgforge-dev/Anylinux-sharun/releases/latest/download/sharun-$ARCH"
  chmod +x "$SHARUN_BIN"
fi

echo "[*] Bundling dependencies with lib4bin..."
DEPLOY_OPENGL=1 DEPLOY_WEBKIT2GTK=1 \
  "$SHARUN_BIN" lib4bin "$APPDIR/usr/bin/$APP" --appdir "$APPDIR"

cp "$SHARUN_BIN" "$APPDIR/AppRun"

DESKTOP_SRC="$(find "$RELEASE_DIR" -name '*.desktop' 2>/dev/null | head -1)"
if [ -n "$DESKTOP_SRC" ]; then
  cp "$DESKTOP_SRC" "$APPDIR/$APP.desktop"
else
  cat > "$APPDIR/$APP.desktop" <<EOF
[Desktop Entry]
Name=$APP
Exec=$APP
Icon=$APP
Type=Application
Categories=Network;RemoteAccess;
EOF
fi

ICON_SRC="$(find "$SCRIPT_DIR/../src-tauri/icons" -name '128x128.png' 2>/dev/null | head -1)"
if [ -n "$ICON_SRC" ]; then
  cp "$ICON_SRC" "$APPDIR/$APP.png"
else
  ICON_SRC="$(find "$SCRIPT_DIR/../src-tauri/icons" -name '*.png' 2>/dev/null | head -1)"
  [ -n "$ICON_SRC" ] && cp "$ICON_SRC" "$APPDIR/$APP.png"
fi

echo "[*] Downloading appimagetool..."
APPIMAGETOOL="$APPDIR/appimagetool"
wget -qO "$APPIMAGETOOL" \
  "https://github.com/AppImage/appimagetool/releases/latest/download/appimagetool-$ARCH.AppImage"
chmod +x "$APPIMAGETOOL" || true

if ! "$APPIMAGETOOL" --help >/dev/null 2>&1; then
  echo "[*] Extracting appimagetool (no FUSE)..."
  cd "$APPDIR" && "$APPIMAGETOOL" --appimage-extract >/dev/null 2>&1
  APPIMAGETOOL="$APPDIR/squashfs-root/AppRun"
  cd "$RELEASE_DIR"
fi

echo "[*] Creating AppImage..."
OUTDIR="$RELEASE_DIR/bundle/appimage"
mkdir -p "$OUTDIR"
OUTPUT="$OUTDIR/${APP}_${VERSION}_${ARCH}.AppImage"
ARCH="$ARCH" "$APPIMAGETOOL" "$APPDIR" "$OUTPUT"

echo "[+] AppImage created: $OUTPUT"
ls -lh "$OUTPUT"

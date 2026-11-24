#!/usr/bin/env bash
# Build LPBrower Launcher (.dmg + .app) natively on macOS and copy deliverables to dist-desktop/
# Usage:
#   bash scripts/build_macos.sh              # Apple Silicon (arm64) or current Mac arch
#   bash scripts/build_macos.sh --universal  # Universal binary (arm64 + x86_64)

set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "Error: macOS bundles (.app / .dmg) require macOS Xcode Command Line Tools (codesign, hdiutil)." >&2
  echo "On Windows/Linux, push to GitHub and run the 'Build LPBrower macOS' GitHub Actions workflow." >&2
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

MODE="${1:-native}"
if [[ "$MODE" == "--universal" ]]; then
  rustup target add aarch64-apple-darwin x86_64-apple-darwin
  TAURI_TARGET="universal-apple-darwin"
  SUFFIX="universal"
else
  HOST_ARCH="$(uname -m)"
  if [[ "$HOST_ARCH" == "arm64" ]]; then
    TAURI_TARGET="aarch64-apple-darwin"
    SUFFIX="arm64"
  else
    TAURI_TARGET="x86_64-apple-darwin"
    SUFFIX="x64"
  fi
  rustup target add "$TAURI_TARGET"
fi

echo "==> Installing npm dependencies..."
if [[ -f ui-kit/package-lock.json ]]; then
  npm --prefix ui-kit ci
else
  npm --prefix ui-kit install
fi
if [[ -f package-lock.json ]]; then
  npm ci
else
  npm install
fi

echo "==> Building LPBrower Launcher for macOS ($TAURI_TARGET)..."
npx tauri build --target "$TAURI_TARGET"

BUNDLE_DIR="src-tauri/target/${TAURI_TARGET}/release/bundle"
APP_PATH="$(find "${BUNDLE_DIR}/macos" -maxdepth 1 -name '*.app' -print -quit)"

if [[ -z "$APP_PATH" ]]; then
  echo "Error: .app bundle not found in ${BUNDLE_DIR}/macos" >&2
  exit 1
fi

echo "==> Applying ad-hoc signature to $(basename "$APP_PATH")..."
xattr -cr "$APP_PATH" || true
codesign --force --deep --sign - "$APP_PATH"
codesign --verify --deep --strict "$APP_PATH"

mkdir -p dist-desktop
rm -rf "dist-desktop/$(basename "$APP_PATH")"
cp -R "$APP_PATH" "dist-desktop/"

STAGE_DMG="$(mktemp -d)"
cp -R "$APP_PATH" "$STAGE_DMG/"
ln -s /Applications "$STAGE_DMG/Applications"
DMG_OUT="dist-desktop/LPBrower_Setup_macOS_${SUFFIX}.dmg"
hdiutil create \
  -volname "LPBrower Launcher" \
  -srcfolder "$STAGE_DMG" \
  -ov \
  -format UDZO \
  "$DMG_OUT"
rm -rf "$STAGE_DMG"

echo ""
echo "==> Done! macOS build artifacts ready in dist-desktop/:"
ls -lh "dist-desktop/$DMG_OUT" 2>/dev/null || ls -lh "$DMG_OUT"
echo "    App bundle: dist-desktop/$(basename "$APP_PATH")"

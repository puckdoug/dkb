#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "==> Building dkb in release mode..."
cargo build --release --manifest-path "${ROOT_DIR}/Cargo.toml"

APP_NAME="Daily Kanban"
BUNDLE_DIR="${ROOT_DIR}/target/release/bundle"
APP_DIR="${BUNDLE_DIR}/${APP_NAME}.app"
CONTENTS_DIR="${APP_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

echo "==> Creating macOS App Bundle structure..."
rm -rf "${APP_DIR}"
mkdir -p "${MACOS_DIR}" "${RESOURCES_DIR}"

echo "==> Copying executable binary..."
cp "${ROOT_DIR}/target/release/dkb" "${MACOS_DIR}/dkb"
chmod +x "${MACOS_DIR}/dkb"

echo "==> Copying Info.plist..."
cp "${ROOT_DIR}/resources/Info.plist" "${CONTENTS_DIR}/Info.plist"

echo "==> Writing PkgInfo..."
printf "APPL????" > "${CONTENTS_DIR}/PkgInfo"

echo "==> Copying AppIcon..."
if [ -f "${ROOT_DIR}/assets/AppIcon.icns" ]; then
    cp "${ROOT_DIR}/assets/AppIcon.icns" "${RESOURCES_DIR}/AppIcon.icns"
else
    echo "Warning: assets/AppIcon.icns not found, skipping icon copy."
fi

# Optional ad-hoc signing if codesign is available
if command -v codesign &>/dev/null; then
    echo "==> Ad-hoc code signing application bundle..."
    codesign --force --deep --sign - "${APP_DIR}"
fi

echo "==> Successfully created '${APP_DIR}'"

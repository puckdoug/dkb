#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

SIGNING_IDENTITY="${APP_SIGN_IDENTITY:-}"
INSTALLER_IDENTITY="${INSTALLER_SIGN_IDENTITY:-}"
PROVISIONING_PROFILE="${PROVISIONING_PROFILE:-}"

APP_NAME="Daily Kanban"
BUNDLE_DIR="${ROOT_DIR}/target/release/bundle"
APP_DIR="${BUNDLE_DIR}/${APP_NAME}.app"
PKG_OUTPUT="${BUNDLE_DIR}/${APP_NAME}.pkg"
ENTITLEMENTS="${ROOT_DIR}/resources/dkb.entitlements"

echo "==> Bundling application first..."
"${SCRIPT_DIR}/bundle_macos.sh"

if [ -z "${SIGNING_IDENTITY}" ]; then
    echo "=========================================================================="
    echo "Notice: APP_SIGN_IDENTITY not set. Using ad-hoc signature for demonstration."
    echo "For Mac App Store or Developer ID export, provide:"
    echo "  export APP_SIGN_IDENTITY=\"3rd Party Mac Developer Application: Developer (TEAMID)\""
    echo "  export INSTALLER_SIGN_IDENTITY=\"3rd Party Mac Developer Installer: Developer (TEAMID)\""
    echo "  export PROVISIONING_PROFILE=\"/path/to/embedded.provisionprofile\""
    echo "=========================================================================="
    SIGN_FLAG="-"
else
    SIGN_FLAG="${SIGNING_IDENTITY}"
fi

if [ -n "${PROVISIONING_PROFILE}" ] && [ -f "${PROVISIONING_PROFILE}" ]; then
    echo "==> Embedding provisioning profile..."
    cp "${PROVISIONING_PROFILE}" "${APP_DIR}/Contents/embedded.provisionprofile"
fi

echo "==> Signing application bundle with entitlements..."
codesign --force --timestamp --options runtime \
    --entitlements "${ENTITLEMENTS}" \
    --sign "${SIGN_FLAG}" \
    "${APP_DIR}"

if command -v productbuild &>/dev/null; then
    echo "==> Building installer package..."
    if [ -n "${INSTALLER_IDENTITY}" ]; then
        productbuild --component "${APP_DIR}" /Applications \
            --sign "${INSTALLER_IDENTITY}" \
            "${PKG_OUTPUT}"
    else
        productbuild --component "${APP_DIR}" /Applications \
            "${PKG_OUTPUT}"
    fi
    echo "==> Created installer package: ${PKG_OUTPUT}"
fi

echo "==> Packaging complete!"

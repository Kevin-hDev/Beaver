#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  exit 0
fi

source scripts/cef-runtime-profile.sh

HELPERS=(
  "Beaver Helper"
  "Beaver Helper (GPU)"
  "Beaver Helper (Renderer)"
  "Beaver Helper (Plugin)"
  "Beaver Helper (Alerts)"
)
SIGNING_IDENTITY="${APPLE_SIGNING_IDENTITY:--}"
DEV_PREP="${CLGO_CEF_DEV_PREP:-0}"
ALLOW_ADHOC_SIGNING="${CLGO_CEF_ALLOW_ADHOC_SIGNING:-0}"
BUILD_FEATURES="${CLGO_CEF_CARGO_FEATURES:-}"
BUILD_TARGET="${CARGO_BUILD_TARGET:-}"
if ! resolve_cef_runtime_profile; then
  echo "CEF build features are invalid" >&2
  exit 1
fi
if [[ -z "$SIGNING_IDENTITY" || ${#SIGNING_IDENTITY} -gt 256 \
  || "$SIGNING_IDENTITY" == *$'\n'* || "$SIGNING_IDENTITY" == *$'\r'* \
  || "$SIGNING_IDENTITY" == *$'\t'* ]]; then
  echo "CEF signing identity is invalid" >&2
  exit 1
fi
if [[ "$DEV_PREP" != "0" && "$DEV_PREP" != "1" ]]; then
  echo "CEF build mode is invalid" >&2
  exit 1
fi
if [[ "$ALLOW_ADHOC_SIGNING" != "0" && "$ALLOW_ADHOC_SIGNING" != "1" ]]; then
  echo "CEF signing mode is invalid" >&2
  exit 1
fi
if [[ -n "$BUILD_FEATURES" && "$BUILD_FEATURES" != "e2e" ]]; then
  echo "CEF build features are invalid" >&2
  exit 1
fi
if [[ -n "$BUILD_TARGET" ]]; then
  if [[ "$BUILD_FEATURES" == "e2e" ]]; then
    echo "CEF E2E cross-target build is invalid" >&2
    exit 1
  fi
  if [[ "$BUILD_TARGET" != "aarch64-apple-darwin" ]]; then
    echo "CEF build target is invalid" >&2
    exit 1
  fi
  TARGET_RELEASE_DIR="target/$BUILD_TARGET/release"
fi
if [[ "$SIGNING_IDENTITY" == "-" && "$DEV_PREP" != "1" \
  && "$ALLOW_ADHOC_SIGNING" != "1" ]]; then
  echo "CEF ad hoc release signing must be explicitly allowed" >&2
  exit 1
fi
ENTITLEMENTS="Entitlements.plist"
if [[ "$DEV_PREP" == "1" ]]; then
  ENTITLEMENTS="Entitlements.dev.plist"
fi

node ../scripts/cef/prepare-cef-source.mjs
BUILD_ARGS=(build --release --bin cl-go-dash-helper)
if [[ -n "$BUILD_FEATURES" ]]; then
  BUILD_ARGS+=(--features "$BUILD_FEATURES")
fi
"${CARGO:-cargo}" "${BUILD_ARGS[@]}"

CEF_DIR=".cef-verified/current"
CEF_FRAMEWORK="$CEF_DIR/Release/Chromium Embedded Framework.framework"
if [[ ! -d "$CEF_FRAMEWORK" \
  || ! -s "$CEF_DIR/LICENSE.txt" ]]; then
  echo "CEF runtime verification failed" >&2
  exit 1
fi

STAGE="$CEF_RUNTIME_STAGE"
rm -rf -- "$STAGE"
mkdir -p "$STAGE"
ditto "$CEF_FRAMEWORK" \
  "$STAGE/Chromium Embedded Framework.framework"
ditto "resources/cef/macos/helpers" "$STAGE/helpers"
install -m 644 "$CEF_DIR/LICENSE.txt" "$STAGE/LICENSE.txt"
if [[ ! -s "$STAGE/LICENSE.txt" ]] || (( $(wc -c < "$STAGE/LICENSE.txt") > 2097152 )); then
  echo "CEF license verification failed" >&2
  exit 1
fi

for library in "$STAGE/Chromium Embedded Framework.framework/Libraries/"*.dylib; do
  if [[ ! -f "$library" ]]; then
    echo "CEF framework validation failed" >&2
    exit 1
  fi
  codesign --force --options runtime --sign "$SIGNING_IDENTITY" "$library"
done
codesign --force --options runtime --sign "$SIGNING_IDENTITY" \
  "$STAGE/Chromium Embedded Framework.framework"

for helper in "${HELPERS[@]}"; do
  app="$STAGE/helpers/$helper.app"
  destination="$STAGE/helpers/$helper.app/Contents/MacOS/$helper"
  mkdir -p "$(dirname "$destination")"
  install -m 755 "$TARGET_RELEASE_DIR/cl-go-dash-helper" "$destination"
  codesign --force --options runtime --entitlements "$ENTITLEMENTS" \
    --sign "$SIGNING_IDENTITY" "$destination"
  codesign --force --options runtime --entitlements "$ENTITLEMENTS" \
    --sign "$SIGNING_IDENTITY" "$app"
done

printf '%s\n' "$CEF_RUNTIME_PROFILE" > "$STAGE/.profile"
touch "$STAGE/.prepared"

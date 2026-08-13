#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  exit 0
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
cd "$ROOT"

source scripts/cef-runtime-profile.sh
if ! resolve_cef_runtime_profile; then
  echo "CEF development runtime validation failed" >&2
  exit 1
fi

HELPER_BINARY="$TARGET_RELEASE_DIR/cl-go-dash-helper"
STAGE="$CEF_RUNTIME_STAGE"
STAMP="$STAGE/.prepared"
PROFILE_MARKER="$STAGE/.profile"
CEF_FRAMEWORK_BINARY=".cef-verified/current/Release/Chromium Embedded Framework.framework/Chromium Embedded Framework"
CEF_LICENSE=".cef-verified/current/LICENSE.txt"
HELPERS=(
  "Beaver Helper"
  "Beaver Helper (GPU)"
  "Beaver Helper (Renderer)"
  "Beaver Helper (Plugin)"
  "Beaver Helper (Alerts)"
)
INPUTS=(
  "Cargo.toml"
  "Cargo.lock"
  "build.rs"
  "Entitlements.dev.plist"
  "scripts/cef-runtime-profile.sh"
  "scripts/prepare-cef.sh"
  "src/bin/cl-go-dash-helper.rs"
)

CACHE_VALID=true
if [[ ! -x "$HELPER_BINARY" \
  || ! -d "$STAGE/Chromium Embedded Framework.framework" \
  || ! -s "$STAGE/LICENSE.txt" \
  || ! -f "$STAMP" ]]; then
  CACHE_VALID=false
fi
if [[ "$CACHE_VALID" == true ]] && ! cef_runtime_profile_matches "$PROFILE_MARKER"; then
  CACHE_VALID=false
fi

for helper in "${HELPERS[@]}"; do
  if [[ ! -x "$STAGE/helpers/$helper.app/Contents/MacOS/$helper" ]]; then
    CACHE_VALID=false
    break
  fi
done

if [[ "$CACHE_VALID" == true ]]; then
  for input in "${INPUTS[@]}" "$HELPER_BINARY" "$CEF_FRAMEWORK_BINARY" "$CEF_LICENSE" \
    resources/cef/macos/helpers/*/Contents/Info.plist; do
    if [[ ! -f "$input" || "$input" -nt "$STAMP" ]]; then
      CACHE_VALID=false
      break
    fi
  done
fi

if [[ "$CACHE_VALID" == true ]]; then
  exit 0
fi

CLGO_CEF_DEV_PREP=1 \
  CLGO_CEF_CARGO_FEATURES="${CLGO_CEF_CARGO_FEATURES:-}" \
  bash scripts/prepare-cef.sh

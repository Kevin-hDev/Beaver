#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  exit 0
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
cd "$ROOT"

HELPER_BINARY="target/release/cl-go-dash-helper"
STAGE="target/cef-runtime/macos"
STAMP="$STAGE/.prepared"
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

CLGO_CEF_DEV_PREP=1 bash scripts/prepare-cef.sh

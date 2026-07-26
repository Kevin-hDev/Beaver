#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_DIR="$(cd -- "$SCRIPT_DIR/../.." && pwd)"
readonly LOCK_DIR="$PROJECT_DIR/src-tauri/resources/forecast-sidecar/runtime-locks"
readonly -a ALL_FAMILIES=(
  chronos
  flowstate
  kairos
  moirai
  sundial
  tabpfn
  timesfm
  tirex
  toto
)
readonly -a PLATFORMS=(macos linux windows)

if ! command -v uv >/dev/null 2>&1; then
  echo "uv est requis pour verrouiller les runtimes Forecast." >&2
  exit 1
fi

if (( $# > 1 )); then
  echo "Usage: npm run forecast:lock-runtimes -- [famille]" >&2
  exit 1
fi

families=("${ALL_FAMILIES[@]}")
if (( $# == 1 )); then
  requested="$1"
  if [[ ! "$requested" =~ ^[a-z0-9-]{1,32}$ ]]; then
    echo "Famille Forecast invalide." >&2
    exit 1
  fi
  supported=false
  for family in "${ALL_FAMILIES[@]}"; do
    if [[ "$family" == "$requested" ]]; then
      supported=true
      break
    fi
  done
  if [[ "$supported" != true ]]; then
    echo "Famille Forecast inconnue." >&2
    exit 1
  fi
  families=("$requested")
fi

for family in "${families[@]}"; do
  for platform in "${PLATFORMS[@]}"; do
    uv pip compile "$LOCK_DIR/$family.in" \
      --output-file "$LOCK_DIR/$family.$platform.lock" \
      --python-version 3.12 \
      --python-platform "$platform" \
      --generate-hashes \
      --only-binary :all: \
      --no-header \
      --no-annotate
  done
done

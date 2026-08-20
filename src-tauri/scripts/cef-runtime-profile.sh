#!/usr/bin/env bash
# shellcheck shell=bash

# Bibliothèque sourcée par les scripts CEF macOS ; ne pas l'exécuter directement.

resolve_cef_runtime_profile() {
  case "${CLGO_CEF_CARGO_FEATURES:-}" in
    "")
      CEF_RUNTIME_PROFILE="dev"
      CEF_TARGET_ROOT="target"
      TARGET_RELEASE_DIR="target/release"
      ;;
    "e2e")
      CEF_RUNTIME_PROFILE="e2e"
      CEF_TARGET_ROOT="target/e2e"
      TARGET_RELEASE_DIR="target/e2e/release"
      ;;
    *)
      return 1
      ;;
  esac
  CEF_RUNTIME_STAGE="$CEF_TARGET_ROOT/cef-runtime/macos"
}

cef_e2e_target_dir_matches() {
  local project_root="$1"
  local target_dir="${CARGO_TARGET_DIR:-}"
  if [[ "$CEF_RUNTIME_PROFILE" != "e2e" ]]; then
    return 0
  fi
  if [[ -z "$target_dir" || ${#target_dir} -gt 4096 \
    || "$target_dir" == *$'\n'* || "$target_dir" == *$'\r'* \
    || "$target_dir" == *$'\t'* ]]; then
    return 1
  fi
  [[ "$target_dir" == "$project_root/$CEF_TARGET_ROOT" ]]
}

cef_runtime_profile_matches() {
  local marker="$1"
  local marker_size
  local observed_profile
  if [[ -L "$marker" || ! -f "$marker" ]]; then
    return 1
  fi
  marker_size="$(wc -c < "$marker")" || return 1
  if (( marker_size < 2 || marker_size > 16 )); then
    return 1
  fi
  observed_profile="$(< "$marker")" || return 1
  [[ "$observed_profile" == "$CEF_RUNTIME_PROFILE" ]]
}

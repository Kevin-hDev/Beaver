#!/usr/bin/env bash

# tauri.conf.json reste l'unique autorité : les manifests CEF sont des gabarits.
load_cef_bundle_version() {
  local config="tauri.conf.json"
  local version
  if [[ ! -f "$config" ]] || ! version="$(
    /usr/bin/plutil -extract version raw -o - "$config" 2>/dev/null
  )"; then
    return 1
  fi
  if [[ ${#version} -gt 32 \
    || ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    return 1
  fi
  CEF_BUNDLE_VERSION="$version"
}

apply_cef_bundle_version() {
  local manifest="$1"
  if [[ -z "${CEF_BUNDLE_VERSION:-}" \
    || -z "$manifest" || ${#manifest} -gt 4096 \
    || "$manifest" == -* || "$manifest" == *$'\n'* \
    || "$manifest" == *$'\r'* || "$manifest" == *$'\t'* \
    || ! -f "$manifest" ]]; then
    return 1
  fi
  /usr/bin/plutil -replace CFBundleShortVersionString \
    -string "$CEF_BUNDLE_VERSION" "$manifest" >/dev/null \
    && /usr/bin/plutil -replace CFBundleVersion \
      -string "$CEF_BUNDLE_VERSION" "$manifest" >/dev/null
}

#!/usr/bin/env bash
# shellcheck shell=bash

# Bibliothèque sourcée par les scripts CEF macOS ; ne pas l'exécuter directement.

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

cef_bundle_version_matches() {
  local marker="$1"
  local marker_size
  local recorded
  if [[ -z "$marker" || ${#marker} -gt 4096 \
    || "$marker" == -* || "$marker" == *$'\n'* \
    || "$marker" == *$'\r'* || "$marker" == *$'\t'* \
    || -L "$marker" || ! -f "$marker" ]]; then
    return 1
  fi
  marker_size="$(wc -c < "$marker")" || return 1
  if (( marker_size < 2 || marker_size > 40 )); then
    return 1
  fi
  if ! load_cef_bundle_version || ! recorded="$(< "$marker")"; then
    return 1
  fi
  [[ "$recorded" == "$CEF_BUNDLE_VERSION" ]]
}

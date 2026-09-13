#!/bin/bash
set -euo pipefail
readonly REPOSITORY="Kevin-hDev/Beaver" MANIFEST_NAME="update-manifest.json"
readonly API_URL="https://api.github.com/repos/${REPOSITORY}/releases/latest"
readonly MAX_API_BYTES=524288 MAX_MANIFEST_BYTES=65536 MAX_ASSET_BYTES=2147483648
readonly CURL="/usr/bin/curl"
TMP_DIR="" RUN_ID=""
info() { printf "\033[1;34m→\033[0m %s\n" "$1"; }
ok() { printf "\033[1;32m✓\033[0m %s\n" "$1"; }
fail() { printf "\033[1;31m✗\033[0m %s\n" "$1" >&2; exit 1; }
valid_version() { [ "${#1}" -le 32 ] && [[ "$1" =~ ^(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})\.(0|[1-9][0-9]{0,8})$ ]]; }
file_size() { LC_ALL=C /usr/bin/wc -c 2>/dev/null < "$1" | /usr/bin/tr -d '[:space:]'; }
redirect_allowed() {
  [ "${#1}" -le 4096 ] || return 1
  case "$1" in https://release-assets.githubusercontent.com/*) return 0 ;; *) return 1 ;; esac
}
header_location() {
  LC_ALL=C /usr/bin/awk '
    tolower($1) == "location:" { value = $2; sub(/\r$/, "", value); count += 1 }
    END { if (count == 1) print value; else exit 1 }
  ' "$1"
}
download_bounded() {
  local current="$1" destination="$2" limit="$3" redirects_allowed="$4" timeout="$5"
  local redirects=0 code="" location="" headers="${TMP_DIR}/headers" part="${destination}.part"
  while :; do
    : > "$headers"
    code=$("$CURL" --silent --proto '=https' --tlsv1.2 \
      --connect-timeout 10 --max-time "$timeout" --max-filesize "$limit" \
      --header "Accept-Encoding: identity" --header "User-Agent: Beaver-Installer/1" \
      --dump-header "$headers" --output "$part" --write-out '%{http_code}' "$current") ||
      return 1
    [ "$(file_size "$headers")" -le 65536 ] || return 1
    if [ "$code" = "200" ]; then
      [ "$(file_size "$part")" -ge 1 ] && [ "$(file_size "$part")" -le "$limit" ] || return 1
      /bin/mv -f "$part" "$destination" 2>/dev/null || return 1
      return 0
    fi
    case "$code" in 301|302|303|307|308) ;; *) return 1 ;; esac
    [ "$redirects_allowed" = "1" ] && [ "$redirects" -lt 3 ] || return 1
    location=$(header_location "$headers") || return 1
    redirect_allowed "$location" || return 1
    current="$location"
    redirects=$((redirects + 1))
  done
}
release_version() {
  local file="$1" version=""
  # L'API GitHub est validée sur les seuls champs attendus, sans exécuter son contenu.
  version=$(/usr/bin/sed -n \
    's/^[[:space:]]*"tag_name":[[:space:]]*"v\([^"]*\)",[[:space:]]*$/\1/p' "$file")
  valid_version "$version" || return 1
  [ "$(/usr/bin/grep -Ec '^[[:space:]]*"draft":[[:space:]]*false,' "$file")" -eq 1 ] || return 1
  [ "$(/usr/bin/grep -Ec '^[[:space:]]*"prerelease":[[:space:]]*false,' "$file")" -eq 1 ] || return 1
  [ "$(/usr/bin/grep -c '"browser_download_url":' "$file")" -le 64 ] || return 1
  printf "%s" "$version"
}
release_has_url() { [ "$(/usr/bin/grep -F -c "\"browser_download_url\": \"$1\"" "$2")" -eq 1 ]; }
manifest_values() {
  # create-update-manifest.mjs produit volontairement ce JSON canonique, lu sans parseur externe.
  LC_ALL=C /usr/bin/awk -v version="$1" -v expected="$2" '
    NR == 1 { if ($0 != "{") exit 2; next }
    NR == 2 { if ($0 != "  \"version\": \"" version "\",") exit 2; next }
    NR == 3 { if ($0 != "  \"assets\": [") exit 2; state = 1; next }
    state == 1 { if ($0 != "    {") exit 2; state = 2; next }
    state == 2 {
      if ($0 !~ /^      "name": "[A-Za-z0-9._-]+",$/) exit 2
      name = $0; sub(/^      "name": "/, "", name); sub(/",$/, "", name)
      if (index(name, "Beaver_" version "_") != 1 || seen[name]++) exit 2; state = 3; next
    }
    state == 3 {
      if ($0 !~ /^      "sha256": "[0-9a-f]+",$/) exit 2
      hash = $0; sub(/^      "sha256": "/, "", hash); sub(/",$/, "", hash)
      if (length(hash) != 64) exit 2; state = 4; next
    }
    state == 4 {
      if ($0 !~ /^      "size": [0-9]+$/) exit 2
      size_text = $0; sub(/^      "size": /, "", size_text); size = size_text + 0
      if (size < 1 || size > 2147483648) exit 2; state = 5; next
    }
    state == 5 {
      count += 1; if (count > 16) exit 2
      if (name == expected) { found += 1; result = hash " " size_text }
      if ($0 == "    },") { state = 1; next }
      if ($0 == "    }") { state = 6; next }
      exit 2
    }
    state == 6 { if ($0 != "  ]") exit 2; state = 7; next }
    state == 7 { if ($0 != "}") exit 2; state = 8; next }
    { exit 2 }
    END { if (state != 8 || count < 1 || found != 1) exit 2; print result }
  ' "$3"
}
sha256_file() {
  if [ "$1" = "macos" ]; then /usr/bin/shasum -a 256 "$2" 2>/dev/null | /usr/bin/awk '{print $1}'
  else /usr/bin/sha256sum "$2" 2>/dev/null | /usr/bin/awk '{print $1}'; fi
}
canonical_dir() { [ -d "$1" ] && (cd "$1" && /bin/pwd -P); }
owned_run_valid() {
  local root="$1" candidate="$2" run_id="$3" marker="$2/.beaver-installer-owner.json" owner=""
  [ -d "$candidate" ] && [ ! -L "$candidate" ] && [ "${candidate##*/}" = "beaver-install-$run_id" ] || return 1
  [ "$(canonical_dir "${candidate%/*}")" = "$(canonical_dir "$root")" ] || return 1
  [ -f "$marker" ] && [ ! -L "$marker" ] && [ "$(file_size "$marker")" -le 128 ] || return 1
  [ "$(/bin/cat "$marker")" = "{\"schema\":1,\"runId\":\"$run_id\"}" ] || return 1
  owner=$(/usr/bin/id -un)
  [ "$(/usr/bin/find -P "$candidate" -print | /usr/bin/awk 'NR > 4097 { exit 1 } END { print NR }')" -le 4097 ] || return 1
  ! /usr/bin/find -P "$candidate" \( -type l -o \( ! -type d ! -type f \) -o ! -user "$owner" \) -print -quit | /usr/bin/grep -q .
}
archive_names_valid() {
  LC_ALL=C /usr/bin/awk '
    { sub(/\/$/, ""); if (++count > 4096 || length($0) > 1024 || $0 ~ /[[:cntrl:]]/ || substr($0,1,1) == "/" || seen[$0]++) exit 1
      parts = split($0, path, "/"); for (i = 1; i <= parts; i++) if (path[i] == "..") exit 1
      if ($0 != "Beaver Installer.app" && index($0, "Beaver Installer.app/") != 1) exit 1 }
    END { if (count < 1) exit 1 }
  '
}
verify_installer_bundle() {
  local app="$1" plist="$1/Contents/Info.plist" executable=""
  [ -d "$app" ] && [ ! -L "$app" ] && [ -f "$plist" ] && [ ! -L "$plist" ] || return 1
  [ "$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$plist" 2>/dev/null)" = "com.clgo.dash.installer" ] || return 1
  executable=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleExecutable' "$plist" 2>/dev/null)
  [[ "$executable" =~ ^[A-Za-z0-9._-]{1,128}$ ]] && [ -f "$app/Contents/MacOS/$executable" ] && [ ! -L "$app/Contents/MacOS/$executable" ]
}
launch_macos() {
  local archive="$1" app_name="$2" app_size="$3" app_hash="$4" extract="$TMP_DIR/unpacked" app=""
  /usr/bin/tar -tzf "$archive" | archive_names_valid || fail "Archive d'installation invalide."
  /usr/bin/tar -tvzf "$archive" | /usr/bin/awk 'NR > 4096 || substr($0,1,1) !~ /[-d]/ { exit 1 }' || fail "Archive d'installation invalide."
  /bin/mkdir "$extract" && COPYFILE_DISABLE=1 /usr/bin/tar -xzf "$archive" -C "$extract" --no-same-owner --no-same-permissions || fail "Archive d'installation invalide."
  app="$extract/Beaver Installer.app"; verify_installer_bundle "$app" || fail "Archive d'installation invalide."
  ! /usr/bin/find -P "$extract" \( -type l -o \( ! -type d ! -type f \) \) -print -quit | /usr/bin/grep -q . || fail "Archive d'installation invalide."
  /usr/bin/open -n "$app" --args --run-id "$RUN_ID" --work-dir "$TMP_DIR" --version "$VERSION" \
    --app-asset-name "$app_name" --app-asset-size "$app_size" --app-asset-sha256 "$app_hash" || fail "Ouverture impossible."
}
run_as_root() {
  if [ "$(/usr/bin/id -u)" -eq 0 ]; then "$@" 2>/dev/null; else
    [ -x /usr/bin/sudo ] || fail "Droits administrateur requis."
    /usr/bin/sudo "$@" 2>/dev/null
  fi
}
package_installed() { /usr/bin/dpkg-query -W -f='${db:Status-Abbrev}' "$1" 2>/dev/null | /usr/bin/grep -q '^ii '; }
install_linux() {
  local asset="$1"
  ! package_installed beaver && ! package_installed cl-go ||
    fail "Une application est déjà installée. Utilise sa mise à jour intégrée."
  [ "$(/usr/bin/dpkg-deb -f "$asset" Package 2>/dev/null)" = "beaver" ] &&
    [ "$(/usr/bin/dpkg-deb -f "$asset" Architecture 2>/dev/null)" = "amd64" ] &&
    [ "$(/usr/bin/dpkg-deb -f "$asset" Version 2>/dev/null)" = "$VERSION" ] &&
    [ "$(/usr/bin/dpkg-deb -f "$asset" Provides 2>/dev/null)" = "cl-go" ] &&
    [ "$(/usr/bin/dpkg-deb -f "$asset" Conflicts 2>/dev/null)" = "cl-go" ] &&
    [ "$(/usr/bin/dpkg-deb -f "$asset" Replaces 2>/dev/null)" = "cl-go" ] ||
    fail "Paquet d'installation invalide."
  run_as_root /usr/bin/apt-get install -y "$asset" >/dev/null || fail "Installation impossible."
  [ -f /usr/bin/cl-go-dash ] && [ ! -L /usr/bin/cl-go-dash ] || fail "Installation impossible."
  /usr/bin/cl-go-dash >/dev/null 2>&1 &
}

cleanup() {
  case "$TMP_DIR" in
    /tmp/beaver-install.*) /bin/rm -rf "$TMP_DIR" 2>/dev/null ;;
    /tmp/beaver-install-*) if owned_run_valid /tmp "$TMP_DIR" "$RUN_ID"; then /bin/rm -rf "$TMP_DIR" 2>/dev/null; fi ;;
  esac
}

main() {
  local os="" arch="" platform="" suffix="" release="" manifest="" asset="" values="" expected_hash="" expected_size="" actual_hash="" app_name="" app_values="" app_hash="" app_size="" app_url=""
  [ -x "$CURL" ] || fail "curl est requis."
  os=$(/usr/bin/uname -s); arch=$(/usr/bin/uname -m)
  case "$os:$arch" in
    Darwin:arm64|Darwin:aarch64) platform="macos"; suffix="_aarch64.dmg" ;;
    Linux:x86_64|Linux:amd64) platform="linux"; suffix="_amd64.deb" ;;
    *) fail "Système non pris en charge." ;;
  esac
  umask 077
  if [ "$platform" = "macos" ]; then
    RUN_ID=$(/usr/bin/od -An -N16 -tx1 /dev/urandom | /usr/bin/tr -d '[:space:]')
    [[ "$RUN_ID" =~ ^[0-9a-f]{32}$ ]] || fail "Installation impossible."
    TMP_DIR="/tmp/beaver-install-$RUN_ID"; /bin/mkdir "$TMP_DIR" || fail "Installation impossible."
    printf '{"schema":1,"runId":"%s"}' "$RUN_ID" > "$TMP_DIR/.beaver-installer-owner.json"
  else
    TMP_DIR=$(/usr/bin/mktemp -d /tmp/beaver-install.XXXXXXXX 2>/dev/null) || fail "Installation impossible."
  fi
  trap cleanup EXIT HUP INT TERM
  release="$TMP_DIR/release.json"; download_bounded "$API_URL" "$release" "$MAX_API_BYTES" 0 30 ||
    fail "Impossible de récupérer la version."
  VERSION=$(release_version "$release") || fail "Version de Beaver invalide."
  app_name="Beaver_${VERSION}${suffix}"
  if [ "$platform" = "macos" ]; then ASSET_NAME="Beaver_${VERSION}_installer-aarch64.tar.gz"; else ASSET_NAME="$app_name"; fi
  ASSET_URL="https://github.com/${REPOSITORY}/releases/download/v${VERSION}/${ASSET_NAME}"
  app_url="https://github.com/${REPOSITORY}/releases/download/v${VERSION}/${app_name}"
  MANIFEST_URL="https://github.com/${REPOSITORY}/releases/download/v${VERSION}/${MANIFEST_NAME}"
  release_has_url "$ASSET_URL" "$release" && release_has_url "$MANIFEST_URL" "$release" &&
    { [ "$platform" != "macos" ] || release_has_url "$app_url" "$release"; } ||
    fail "Version de Beaver incomplète."
  manifest="$TMP_DIR/$MANIFEST_NAME"; asset="$TMP_DIR/$ASSET_NAME"
  download_bounded "$MANIFEST_URL" "$manifest" "$MAX_MANIFEST_BYTES" 1 30 ||
    fail "Manifeste de mise à jour invalide."
  values=$(manifest_values "$VERSION" "$ASSET_NAME" "$manifest") ||
    fail "Manifeste de mise à jour invalide."
  if [ "$platform" = "macos" ]; then
    app_values=$(manifest_values "$VERSION" "$app_name" "$manifest") || fail "Manifeste de mise à jour invalide."
    app_hash=${app_values%% *}; app_size=${app_values#* }
  fi
  expected_hash=${values%% *}; expected_size=${values#* }
  info "Téléchargement de Beaver v${VERSION}..."
  download_bounded "$ASSET_URL" "$asset" "$MAX_ASSET_BYTES" 1 1800 ||
    fail "Téléchargement impossible."
  [ "$(file_size "$asset")" = "$expected_size" ] || fail "Téléchargement invalide."
  actual_hash=$(sha256_file "$platform" "$asset") || fail "Téléchargement invalide."
  [ "$actual_hash" = "$expected_hash" ] || fail "Téléchargement invalide."
  if [ "$platform" = "macos" ]; then
    launch_macos "$asset" "$app_name" "$app_size" "$app_hash"
    trap - EXIT HUP INT TERM
    printf "%s\n" "L'installateur Beaver s'est ouvert."
    return
  fi
  info "Installation de Beaver v${VERSION}..."; install_linux "$asset"
  ok "Beaver v${VERSION} est installé."
}

if [ "${BEAVER_INSTALLER_TEST_MODE:-0}" != "1" ]; then main "$@"; fi

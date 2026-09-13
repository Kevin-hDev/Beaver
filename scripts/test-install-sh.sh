#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export BEAVER_INSTALLER_TEST_MODE=1

# shellcheck source=../install.sh
. "${ROOT_DIR}/install.sh"
export BEAVER_DEB_CHECK_TEST_MODE=1
# shellcheck source=release/check-deb-migration.sh
. "${ROOT_DIR}/scripts/release/check-deb-migration.sh"

assert_eq() {
  local expected="$1"
  local actual="$2"
  local label="$3"

  if [ "${actual}" != "${expected}" ]; then
    printf "FAIL %s: expected [%s], got [%s]\n" "${label}" "${expected}" "${actual}" >&2
    exit 1
  fi
}

assert_eq "0" "$(valid_version 1.1.0; printf '%s' "$?")" "strict version"
if valid_version 01.1.0 || valid_version 1.1 || valid_version "../1.1.0"; then
  printf "FAIL invalid version accepted\n" >&2
  exit 1
fi

redirect_allowed "https://release-assets.githubusercontent.com/github-production-release-asset/test"
if redirect_allowed "https://release-assets.githubusercontent.com.evil.test/file" ||
  redirect_allowed "http://release-assets.githubusercontent.com/file"; then
  printf "FAIL invalid redirect accepted\n" >&2
  exit 1
fi

TMP_DIR="$(/usr/bin/mktemp -d /tmp/beaver-install-test.XXXXXXXX)"
MANIFEST="${TMP_DIR}/update-manifest.json"
printf '%s\n' \
  '{' \
  '  "version": "1.1.0",' \
  '  "assets": [' \
  '    {' \
  '      "name": "Beaver_1.1.0_aarch64.dmg",' \
  '      "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",' \
  '      "size": 123456789' \
  '    }' \
  '  ]' \
  '}' > "${MANIFEST}"

assert_eq \
  "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa 123456789" \
  "$(manifest_values 1.1.0 Beaver_1.1.0_aarch64.dmg "${MANIFEST}")" \
  "manifest entry"

if manifest_values 1.1.1 Beaver_1.1.0_aarch64.dmg "${MANIFEST}" >/dev/null 2>&1 ||
  manifest_values 1.1.0 Beaver_1.1.0_amd64.deb "${MANIFEST}" >/dev/null 2>&1; then
  printf "FAIL invalid manifest accepted\n" >&2
  exit 1
fi

printf '%s\n' "Beaver Installer.app/" "Beaver Installer.app/Contents/" | archive_names_valid
if printf '%s\n' "../Beaver Installer.app" | archive_names_valid ||
  printf '%s\n' "Beaver Installer.app" "Beaver Installer.app/" | archive_names_valid; then
  printf "FAIL unsafe archive names accepted\n" >&2
  exit 1
fi

PURGE_ROOT="${TMP_DIR}/purge"
VALID_ID="11111111111111111111111111111111"
FORGED_ID="22222222222222222222222222222222"
LINK_ID="33333333333333333333333333333333"
/bin/mkdir -p "${PURGE_ROOT}/beaver-install-${VALID_ID}" \
  "${PURGE_ROOT}/beaver-install-${FORGED_ID}" "${PURGE_ROOT}/beaver-install-${LINK_ID}"
printf '{"schema":1,"runId":"%s"}' "${VALID_ID}" > "${PURGE_ROOT}/beaver-install-${VALID_ID}/.beaver-installer-owner.json"
printf '{"schema":1,"runId":"wrong"}' > "${PURGE_ROOT}/beaver-install-${FORGED_ID}/.beaver-installer-owner.json"
printf '{"schema":1,"runId":"%s"}' "${LINK_ID}" > "${PURGE_ROOT}/beaver-install-${LINK_ID}/.beaver-installer-owner.json"
SENTINEL="${TMP_DIR}/sentinel"; printf "outside\n" > "${SENTINEL}"
/bin/ln -s "${SENTINEL}" "${PURGE_ROOT}/beaver-install-${LINK_ID}/sentinel-link"
purge_orphans "${PURGE_ROOT}" "ffffffffffffffffffffffffffffffff"
if [ -e "${PURGE_ROOT}/beaver-install-${VALID_ID}" ] ||
  [ ! -e "${PURGE_ROOT}/beaver-install-${FORGED_ID}" ] ||
  [ ! -L "${PURGE_ROOT}/beaver-install-${LINK_ID}/sentinel-link" ] ||
  [ "$(/bin/cat "${SENTINEL}")" != "outside" ]; then
  printf "FAIL safe orphan purge contract\n" >&2
  exit 1
fi

if ! control_contract_matches "beaver" "cl-go" "cl-go" "cl-go"; then
  printf "FAIL valid Debian migration contract was rejected\n" >&2
  exit 1
fi

if control_contract_matches "beaver" "cl-go" "other" "cl-go"; then
  printf "FAIL invalid Debian migration contract was accepted\n" >&2
  exit 1
fi

for prefix in "" "./"; do
  if ! printf '%s\n' \
    "-rwxr-xr-x root/root usr/bin/unrelated" \
    "-rwxr-xr-x root/root ${prefix}usr/bin/cl-go-dash" \
    "-rw-r--r-- root/root ${prefix}usr/share/applications/Beaver.desktop" |
    validate_content_listing; then
    printf "FAIL valid Debian content listing was rejected\n" >&2
    exit 1
  fi
done

if ! {
  awk 'BEGIN { for (i = 0; i < 5000; i += 1) print "-rw-r--r-- root/root usr/share/beaver/file-" i }'
  printf '%s\n' \
    "-rwxr-xr-x root/root usr/bin/cl-go-dash" \
    "-rw-r--r-- root/root usr/share/applications/Beaver.desktop"
} | validate_content_listing; then
  printf "FAIL bounded large Debian content listing was rejected\n" >&2
  exit 1
fi

if {
  awk 'BEGIN { for (i = 0; i < 20001; i += 1) print "-rw-r--r-- root/root usr/share/beaver/file-" i }'
  printf '%s\n' \
    "-rwxr-xr-x root/root usr/bin/cl-go-dash" \
    "-rw-r--r-- root/root usr/share/applications/Beaver.desktop"
} | validate_content_listing; then
  printf "FAIL oversized Debian content listing was accepted\n" >&2
  exit 1
fi

if printf '%s\n' \
  "-rwxr-xr-x root/root usr/bin/cl-go-dash" \
  "-rwxr-xr-x root/root usr/bin/cl-go-dash" \
  "-rw-r--r-- root/root usr/share/applications/Beaver.desktop" |
  validate_content_listing; then
  printf "FAIL duplicate Debian binary was accepted\n" >&2
  exit 1
fi

if printf '%s\n' \
  "-rwxr-xr-x root/root ../usr/bin/cl-go-dash" \
  "-rw-r--r-- root/root usr/share/applications/Beaver.desktop" |
  validate_content_listing; then
  printf "FAIL unsafe Debian content listing was accepted\n" >&2
  exit 1
fi

/bin/rm -rf "${TMP_DIR}"
TMP_DIR=""
printf "install.sh tests OK\n"

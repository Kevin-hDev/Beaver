#!/bin/bash
set -euo pipefail

readonly MAX_PACKAGE_BYTES=2147483648
readonly MAX_CONTENT_ENTRIES=20000

fail() {
  printf 'Debian package validation failed.\n' >&2
  exit 1
}

control_contract_matches() {
  [ "$1" = "beaver" ] &&
    [ "$2" = "cl-go" ] &&
    [ "$3" = "cl-go" ] &&
    [ "$4" = "cl-go" ]
}

read_control_field() {
  local asset="$1"
  local field="$2"
  local value
  value="$(dpkg-deb -f "${asset}" "${field}" | head -c 1025)" || return 1
  [ "${#value}" -le 1024 ] || return 1
  printf '%s' "${value}"
}

validate_content_listing() {
  awk -v max_entries="${MAX_CONTENT_ENTRIES}" '
    NR > max_entries { overflow = 1; next }
    {
      path = $NF
      sub(/^\.\//, "", path)
    }
    path == "usr/bin/cl-go-dash" { binary += 1 }
    path == "usr/share/applications/Beaver.desktop" { desktop += 1 }
    END { exit (overflow || binary != 1 || desktop != 1) }
  '
}

validate_contents() {
  dpkg-deb -c "$1" | validate_content_listing
}

main() {
  [ "$#" -eq 1 ] || fail
  local asset="$1"
  case "${asset}" in
    /*.deb) ;;
    *) fail ;;
  esac
  case "${asset}" in
    *..* | *$'\n'* | *$'\r'*) fail ;;
  esac
  [ -f "${asset}" ] && [ ! -L "${asset}" ] || fail
  [ "$(stat -c '%s' "${asset}")" -le "${MAX_PACKAGE_BYTES}" ] || fail
  command -v dpkg-deb >/dev/null 2>&1 || fail

  local package provides conflicts replaces
  package="$(read_control_field "${asset}" "Package")" || fail
  provides="$(read_control_field "${asset}" "Provides")" || fail
  conflicts="$(read_control_field "${asset}" "Conflicts")" || fail
  replaces="$(read_control_field "${asset}" "Replaces")" || fail

  # Package must be beaver; compatibility fields must be exactly cl-go.
  control_contract_matches \
    "${package}" "${provides}" "${conflicts}" "${replaces}" || fail
  validate_contents "${asset}" || fail
}

if [ "${BEAVER_DEB_CHECK_TEST_MODE:-0}" != "1" ]; then
  main "$@"
fi

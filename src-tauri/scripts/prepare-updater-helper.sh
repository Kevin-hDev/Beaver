#!/usr/bin/env bash
set -euo pipefail

target="${CARGO_BUILD_TARGET:-}"
if [[ ${#target} -gt 128 || ( -n "$target" && ! "$target" =~ ^[A-Za-z0-9_.-]+$ ) ]]; then
  echo "invalid updater target" >&2
  exit 1
fi

cargo_args=(build --release --bin cl-go-dash-updater)
target_dir="target"
extension=""
if [[ -n "$target" ]]; then
  cargo_args+=(--target "$target")
  target_dir="target/$target"
fi
if [[ "$target" == *-windows-* ]] || [[ -z "$target" && "$OSTYPE" == msys* ]]; then
  extension=".exe"
fi

cargo "${cargo_args[@]}"

source_path="$target_dir/release/cl-go-dash-updater$extension"
destination_dir="target/updater-helper"
destination="$destination_dir/cl-go-dash-updater$extension"
mkdir -p "$destination_dir"
if [[ ! -f "$source_path" || -L "$source_path" || ! -s "$source_path" ]]; then
  echo "updater helper build failed" >&2
  exit 1
fi
size="$(wc -c < "$source_path" | tr -d '[:space:]')"
if [[ ! "$size" =~ ^[0-9]+$ || "$size" -eq 0 || "$size" -gt 67108864 ]]; then
  echo "updater helper size invalid" >&2
  exit 1
fi

if [[ "$extension" == ".exe" ]]; then
  rm -f "$destination_dir/cl-go-dash-updater"
else
  rm -f "$destination_dir/cl-go-dash-updater.exe"
fi
if [[ -f "$destination" && ! -L "$destination" ]] \
  && cmp -s "$source_path" "$destination"; then
  chmod 700 "$destination"
  exit 0
fi

temporary="$(mktemp "$destination_dir/.cl-go-dash-updater.XXXXXXXX")"
trap 'rm -f "$temporary"' EXIT
cp "$source_path" "$temporary"
chmod 700 "$temporary"
if ! cmp -s "$source_path" "$temporary"; then
  echo "updater helper copy failed" >&2
  exit 1
fi
mv -f "$temporary" "$destination"
trap - EXIT

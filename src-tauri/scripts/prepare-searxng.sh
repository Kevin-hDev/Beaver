#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ARCHIVE="$ROOT/resources/searxng-sidecar/source.tar.gz"
SOURCE="$ROOT/resources/searxng-sidecar/source"
SOURCE_TMP=""
WHEELS="$ROOT/resources/searxng-sidecar/wheels"
TMP="$WHEELS.tmp"
STAMP="$WHEELS/.requirements.sha256"
LIMIT_BYTES=$((150 * 1024 * 1024))

cleanup() {
  if [[ -n "$SOURCE_TMP" ]]; then
    rm -r "$SOURCE_TMP"
  fi
}
trap cleanup EXIT

if command -v uv >/dev/null 2>&1; then
  PYTHON=(uv run python3)
elif command -v python3 >/dev/null 2>&1; then
  PYTHON=(python3)
elif command -v python >/dev/null 2>&1; then
  PYTHON=(python)
elif command -v py >/dev/null 2>&1; then
  PYTHON=(py -3)
else
  echo "Python introuvable pour preparer SearXNG" >&2
  exit 1
fi

extract_archive() {
  local archive="$1"
  local dest="$2"

  "${PYTHON[@]}" - "$archive" "$dest" <<'PY'
import shutil
import sys
import tarfile
from pathlib import Path, PurePosixPath

archive = Path(sys.argv[1])
dest = Path(sys.argv[2])


def is_metadata(name: str) -> bool:
    return any(
        part.startswith("._")
        or part in {".DS_Store", ".AppleDouble", "__MACOSX", ".py"}
        for part in PurePosixPath(name).parts
    )


with tarfile.open(archive, "r:gz") as tar:
    for member in tar:
        path = PurePosixPath(member.name)
        if path.is_absolute() or ".." in path.parts:
            raise SystemExit(f"Chemin SearXNG dangereux: {member.name}")
        if is_metadata(member.name):
            continue
        target = dest.joinpath(*path.parts)
        if member.isdir():
            target.mkdir(parents=True, exist_ok=True)
        elif member.isfile():
            target.parent.mkdir(parents=True, exist_ok=True)
            source = tar.extractfile(member)
            if source is None:
                raise SystemExit(f"Fichier SearXNG illisible: {member.name}")
            with source, target.open("wb") as output:
                shutil.copyfileobj(source, output)
PY
}

if [[ ! -d "$SOURCE" && -f "$ARCHIVE" ]]; then
  SOURCE_TMP="$(mktemp -d)"
  extract_archive "$ARCHIVE" "$SOURCE_TMP"
  SOURCE="$SOURCE_TMP/source"
fi

if [[ ! -f "$SOURCE/requirements.txt" || ! -f "$SOURCE/setup.py" ]]; then
  echo "SearXNG bundle incomplet" >&2
  exit 1
fi

if find "$SOURCE" \( -name '._*' -o -name '.DS_Store' -o -name '.AppleDouble' -o -name '__MACOSX' -o -name '.py' \) -print -quit | grep -q .; then
  echo "SearXNG bundle pollue par des metadonnees macOS" >&2
  exit 1
fi

HASH="$("${PYTHON[@]}" - "$SOURCE/requirements.txt" "$SOURCE/setup.py" <<'PY'
import hashlib
import sys

outer = hashlib.sha256()
for path in sys.argv[1:]:
    digest = hashlib.sha256()
    with open(path, "rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    outer.update(digest.digest())

print(outer.hexdigest())
PY
)"
if [[ -f "$STAMP" && "$(cat "$STAMP")" == "$HASH" ]] && compgen -G "$WHEELS/*.whl" >/dev/null; then
  exit 0
fi

rm -r "$TMP" 2>/dev/null || true
mkdir -p "$TMP"
"${PYTHON[@]}" -m pip download \
  --only-binary=:all: \
  --dest "$TMP" \
  -r "$SOURCE/requirements.txt"
"${PYTHON[@]}" -m pip download \
  --only-binary=:all: \
  --dest "$TMP" \
  setuptools wheel

SIZE="$(tar -czf - -C "$TMP" . | wc -c | tr -d ' ')"
rm -r "$WHEELS" 2>/dev/null || true
mkdir -p "$WHEELS"

if [[ "$SIZE" -gt "$LIMIT_BYTES" ]]; then
  echo "download-at-first-launch" > "$WHEELS/.mode"
  rm -r "$TMP" 2>/dev/null || true
  exit 0
fi

mv "$TMP"/*.whl "$WHEELS"/
echo "$HASH" > "$STAMP"
rm -r "$TMP" 2>/dev/null || true

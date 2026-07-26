#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TAURI_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
DEFAULT_OLLAMA_VERSION="$(tr -d '[:space:]' < "${TAURI_DIR}/ollama-version.txt")"
OLLAMA_VERSION="${OLLAMA_VERSION:-${DEFAULT_OLLAMA_VERSION}}"
if [[ ! "${OLLAMA_VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
  echo "✗ Version Ollama invalide" >&2
  exit 1
fi
OS="${1:-$(uname -s | tr '[:upper:]' '[:lower:]')}"
ARCH="${2:-$(uname -m)}"

case "${ARCH}" in
  x86_64|amd64) ARCH="amd64" ;;
  aarch64|arm64) ARCH="arm64" ;;
esac

DEST="${TAURI_DIR}/resources/ollama-bundle"
BASE_URL="https://github.com/ollama/ollama/releases/download/v${OLLAMA_VERSION}"
SHA_URL="${BASE_URL}/sha256sum.txt"
TMP_EXTRACT=$(mktemp -d)

rm -rf "${DEST}"
mkdir -p "${DEST}"

case "${OS}" in
  darwin)     ARCHIVE="ollama-darwin.tgz" ;;
  linux)      ARCHIVE="ollama-linux-${ARCH}.tar.zst" ;;
  windows|mingw*|msys*|cygwin*) ARCHIVE="ollama-windows-${ARCH}.zip" ;;
  *)          echo "✗ OS non supporté : ${OS}" >&2; exit 1 ;;
esac

echo "→ Téléchargement Ollama v${OLLAMA_VERSION} (${OS}/${ARCH})..."
curl -fSL --progress-bar -o "/tmp/${ARCHIVE}" "${BASE_URL}/${ARCHIVE}"

echo "→ Vérification intégrité..."
EXPECTED=$(curl -fsSL "${SHA_URL}" | grep "${ARCHIVE}" | awk '{print $1}')
if [ -n "${EXPECTED}" ]; then
  if command -v sha256sum &>/dev/null; then
    ACTUAL=$(sha256sum "/tmp/${ARCHIVE}" | awk '{print $1}')
  else
    ACTUAL=$(shasum -a 256 "/tmp/${ARCHIVE}" | awk '{print $1}')
  fi
  if [ "${ACTUAL}" != "${EXPECTED}" ]; then
    echo "✗ SHA256 invalide" >&2
    rm -f "/tmp/${ARCHIVE}"
    exit 1
  fi
  echo "✓ SHA256 OK"
fi

echo "→ Extraction (bundle complet avec libs GPU)..."
case "${OS}" in
  darwin)
    tar -xzf "/tmp/${ARCHIVE}" -C "${TMP_EXTRACT}"
    ;;
  linux)
    if command -v zstd &>/dev/null; then
      zstd -d "/tmp/${ARCHIVE}" --stdout | tar -x -C "${TMP_EXTRACT}"
    else
      tar --zstd -xf "/tmp/${ARCHIVE}" -C "${TMP_EXTRACT}"
    fi
    ;;
  windows|mingw*|msys*|cygwin*)
    unzip -o -q "/tmp/${ARCHIVE}" -d "${TMP_EXTRACT}"
    ;;
esac

# Déplacer le contenu vers DEST (gère les archives avec ou sans dossier parent)
INNER=$(find "${TMP_EXTRACT}" -maxdepth 1 -mindepth 1)
INNER_COUNT=$(echo "${INNER}" | wc -l | tr -d ' ')
if [ "${INNER_COUNT}" = "1" ] && [ -d "${INNER}" ]; then
  cp -Rf "${INNER}/"* "${DEST}/" 2>/dev/null || true
  cp -Rf "${INNER}/".[!.]* "${DEST}/" 2>/dev/null || true
else
  cp -Rf "${TMP_EXTRACT}/"* "${DEST}/" 2>/dev/null || true
fi

rm -rf "${TMP_EXTRACT}" "/tmp/${ARCHIVE}"

if [ "${OS}" != "windows" ] && [ -f "${DEST}/ollama" ]; then
  chmod +x "${DEST}/ollama"
fi

SIZE=$(du -sh "${DEST}" | awk '{print $1}')
echo "✓ Ollama v${OLLAMA_VERSION} (${OS}/${ARCH}) → ${DEST}/ (${SIZE})"

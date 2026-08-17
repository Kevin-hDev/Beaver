#!/usr/bin/env bash
set -euo pipefail

if [[ "${1:-}" != "--confirm-disposable-profile" || "${2:-}" != "--confirm-data-dir" || "$#" -ne 2 ]]; then
  echo "Usage: native-upgrade-linux.sh --confirm-disposable-profile --confirm-data-dir" >&2
  exit 2
fi

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../.." && pwd -P)"
DATA_DIR="$HOME/.local/share/cl-go-dash"
MODELS_DIR="$HOME/.ollama/models"
PROOF_DIR="$REPO_ROOT/docs/superpowers/validation/native-upgrade-proofs"

[[ -d "$DATA_DIR" ]] || { echo "Disposable profile data directory is missing" >&2; exit 2; }
CANONICAL_DATA_DIR="$(cd -- "$DATA_DIR" && pwd -P)"
CANONICAL_MODELS_DIR="<created by step 2>"
if [[ -d "$MODELS_DIR" ]]; then CANONICAL_MODELS_DIR="$(cd -- "$MODELS_DIR" && pwd -P)"; fi
mkdir -p -- "$PROOF_DIR"

echo "Profile: disposable Linux account or VM only"
echo "Data directory: $CANONICAL_DATA_DIR"
echo "Models directory: $CANONICAL_MODELS_DIR"
[[ "$(basename -- "$CANONICAL_DATA_DIR")" == "cl-go-dash" ]] || { echo "Unexpected data directory" >&2; exit 2; }
read -r -p "Confirm this is a disposable profile (type DISPOSABLE): " PROFILE_CONFIRM
[[ "$PROFILE_CONFIRM" == "DISPOSABLE" ]] || { echo "Disposable profile confirmation refused" >&2; exit 2; }

echo "1. Verify and manually install Beaver v1.1.2."
node "$SCRIPT_DIR/verify-native-upgrade-assets.mjs" --manifest-only
read -r -p "Confirm Beaver v1.1.2 is installed (type BASELINE): " BASELINE_CONFIRM
[[ "$BASELINE_CONFIRM" == "BASELINE" ]] || { echo "Baseline confirmation refused" >&2; exit 2; }
echo "2. Start v1.1.2, download one real local model, and run one local inference."
read -r -p "When complete, press Enter to continue: " _
echo "3. Capture model list, storage hashes, and successful local inference."
node "$SCRIPT_DIR/native-upgrade-proof.mjs" --data-dir "$DATA_DIR" --models-dir "$MODELS_DIR" --confirm-disposable-profile --confirm-data-dir --output "$PROOF_DIR/linux-before.json"
echo "4. Block the network and disable further Ollama model downloads."
read -r -p "Confirm the network is blocked (type OFFLINE): " OFFLINE_CONFIRM
[[ "$OFFLINE_CONFIRM" == "OFFLINE" ]] || { echo "Offline confirmation refused" >&2; exit 2; }
echo "5. Install the branch build manually over Beaver v1.1.2."
read -r -p "Confirm the controlled install (type INSTALL-BEAVER-1.1.2): " INSTALL_CONFIRM
[[ "$INSTALL_CONFIRM" == "INSTALL-BEAVER-1.1.2" ]] || { echo "Install confirmation refused" >&2; exit 2; }
echo "6. Capture the same model list, storage hashes, and local inference without download."
node "$SCRIPT_DIR/native-upgrade-proof.mjs" --data-dir "$DATA_DIR" --models-dir "$MODELS_DIR" --confirm-disposable-profile --confirm-data-dir --output "$PROOF_DIR/linux-after.json"
echo "7. Interrupt the controlled update at its documented cutpoint, relaunch Beaver, await recovery, and infer again."
read -r -p "When recovery is complete, press Enter to continue: " _
node "$SCRIPT_DIR/native-upgrade-proof.mjs" --data-dir "$DATA_DIR" --models-dir "$MODELS_DIR" --confirm-disposable-profile --confirm-data-dir --output "$PROOF_DIR/linux-recovered.json"
echo "8. Close Beaver through its coordinated path and verify no owned Ollama process remains."
read -r -p "After the process check, press Enter to finish: " _
echo "Linux native upgrade proof metadata written under the local validation directory."

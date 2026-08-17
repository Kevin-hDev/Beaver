[CmdletBinding()]
param(
  [switch]$ConfirmDisposableProfile,
  [switch]$ConfirmDataDir
)

if (-not $ConfirmDisposableProfile -or -not $ConfirmDataDir) {
  Write-Error "Usage: native-upgrade-windows.ps1 -ConfirmDisposableProfile -ConfirmDataDir"
  exit 2
}

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = (Resolve-Path (Join-Path $ScriptDir "../..")).Path
$DataDir = Join-Path $env:USERPROFILE ".local/share/cl-go-dash"
$ModelsDir = Join-Path $env:USERPROFILE ".ollama/models"
$ProofDir = Join-Path $RepoRoot "docs/superpowers/validation/native-upgrade-proofs"

if (-not (Test-Path -LiteralPath $DataDir -PathType Container)) {
  Write-Error "Disposable profile data directory is missing"
  exit 2
}
$CanonicalDataDir = (Resolve-Path -LiteralPath $DataDir).Path
$CanonicalModelsDir = "<created by step 2>"
if (Test-Path -LiteralPath $ModelsDir -PathType Container) { $CanonicalModelsDir = (Resolve-Path -LiteralPath $ModelsDir).Path }
New-Item -ItemType Directory -Force -Path $ProofDir | Out-Null

Write-Host "Profile: disposable Windows account or VM only"
Write-Host "Data directory: $CanonicalDataDir"
Write-Host "Models directory: $CanonicalModelsDir"
if ((Split-Path -Leaf $CanonicalDataDir) -ne "cl-go-dash") { Write-Error "Unexpected data directory"; exit 2 }
if ((Read-Host "Confirm this is a disposable profile (type DISPOSABLE)") -ne "DISPOSABLE") { Write-Error "Disposable profile confirmation refused"; exit 2 }

Write-Host "1. Verify and manually install Beaver v1.1.2."
& node (Join-Path $ScriptDir "verify-native-upgrade-assets.mjs") --manifest-only
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
if ((Read-Host "Confirm Beaver v1.1.2 is installed (type BASELINE)") -ne "BASELINE") { Write-Error "Baseline confirmation refused"; exit 2 }
Write-Host "2. Start v1.1.2, download one real local model, and run one local inference."
[void](Read-Host "When complete, press Enter to continue")
Write-Host "3. Capture model list, storage hashes, and successful local inference."
& node (Join-Path $ScriptDir "native-upgrade-proof.mjs") --data-dir $DataDir --models-dir $ModelsDir --confirm-disposable-profile --confirm-data-dir --output (Join-Path $ProofDir "windows-before.json")
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host "4. Block the network and disable further Ollama model downloads."
if ((Read-Host "Confirm the network is blocked (type OFFLINE)") -ne "OFFLINE") { Write-Error "Offline confirmation refused"; exit 2 }
Write-Host "5. Install the branch build manually over Beaver v1.1.2."
if ((Read-Host "Confirm the controlled install (type INSTALL-BEAVER-1.1.2)") -ne "INSTALL-BEAVER-1.1.2") { Write-Error "Install confirmation refused"; exit 2 }
Write-Host "6. Capture the same model list, storage hashes, and local inference without download."
& node (Join-Path $ScriptDir "native-upgrade-proof.mjs") --data-dir $DataDir --models-dir $ModelsDir --confirm-disposable-profile --confirm-data-dir --output (Join-Path $ProofDir "windows-after.json")
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host "7. Interrupt the controlled update at its documented cutpoint, relaunch Beaver, await recovery, and infer again."
[void](Read-Host "When recovery is complete, press Enter to continue")
& node (Join-Path $ScriptDir "native-upgrade-proof.mjs") --data-dir $DataDir --models-dir $ModelsDir --confirm-disposable-profile --confirm-data-dir --output (Join-Path $ProofDir "windows-recovered.json")
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host "8. Close Beaver through its coordinated path and verify no owned Ollama process remains."
[void](Read-Host "After the process check, press Enter to finish")
Write-Host "Windows native upgrade proof metadata written under the local validation directory."

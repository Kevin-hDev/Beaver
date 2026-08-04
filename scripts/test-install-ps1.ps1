$ErrorActionPreference = "Stop"

$paths = @(
    "..\install.ps1",
    "release\check-nsis-migration.ps1",
    "release\check-nsis-migration.test.ps1",
    "release\windows-artifact-helpers.ps1"
)
foreach ($relativePath in $paths) {
    $tokens = $null
    $errors = $null
    $path = (Resolve-Path (Join-Path $PSScriptRoot $relativePath)).Path
    [void][System.Management.Automation.Language.Parser]::ParseFile(
        $path,
        [ref]$tokens,
        [ref]$errors
    )
    if ($errors.Count -ne 0) {
        Write-Error "PowerShell syntax invalid"
        exit 1
    }
}

Write-Host "PowerShell syntax OK"

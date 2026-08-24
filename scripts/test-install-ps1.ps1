$ErrorActionPreference = "Stop"

$paths = @(
    (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..\install.ps1")).Path
)
$releaseScripts = Get-ChildItem -LiteralPath (Join-Path $PSScriptRoot "release") `
    -Filter "*.ps1" -File | Sort-Object FullName
$paths += @($releaseScripts | ForEach-Object { $_.FullName })

foreach ($path in $paths) {
    $tokens = $null
    $errors = $null
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

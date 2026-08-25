[CmdletBinding()]
param(
    [switch]$ListOnly
)

$ErrorActionPreference = "Stop"
$MaxPowerShellScripts = 256
$repositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
$repositoryPrefix = $repositoryRoot.TrimEnd(
    [IO.Path]::DirectorySeparatorChar,
    [IO.Path]::AltDirectorySeparatorChar
) + [IO.Path]::DirectorySeparatorChar

# Git is the authority for release source inventory; disabling quotePath preserves non-ASCII names.
$relativePaths = @(& git -C $repositoryRoot -c core.quotepath=false ls-files -- "*.ps1")
if ($LASTEXITCODE -ne 0 -or $relativePaths.Count -le 0 -or
    $relativePaths.Count -gt $MaxPowerShellScripts) {
    throw "PowerShell syntax inventory failed."
}

$paths = foreach ($relativePath in $relativePaths) {
    if (
        [string]::IsNullOrWhiteSpace($relativePath) -or
        $relativePath.Length -gt 4096 -or
        [IO.Path]::IsPathRooted($relativePath) -or
        $relativePath -match "(^|[\\/])\.\.([\\/]|$)"
    ) {
        throw "PowerShell syntax inventory failed."
    }
    $path = [IO.Path]::GetFullPath((Join-Path $repositoryRoot $relativePath))
    if (-not $path.StartsWith($repositoryPrefix, [StringComparison]::OrdinalIgnoreCase) -or
        -not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "PowerShell syntax inventory failed."
    }
    $path
}

if ($ListOnly) {
    $paths
    return
}

foreach ($path in $paths) {
    $tokens = $null
    $errors = $null
    [void][System.Management.Automation.Language.Parser]::ParseFile(
        $path,
        [ref]$tokens,
        [ref]$errors
    )
    if ($errors.Count -ne 0) {
        throw "PowerShell syntax invalid."
    }
}

Write-Host "PowerShell syntax OK"

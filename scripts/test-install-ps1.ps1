[CmdletBinding()]
param(
    [switch]$ListOnly,
    [string]$RepositoryRoot = ""
)

$ErrorActionPreference = "Stop"
$MaxPowerShellScripts = 256
$repositoryRoot = if ([string]::IsNullOrWhiteSpace($RepositoryRoot)) {
    [IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
} else {
    [IO.Path]::GetFullPath($RepositoryRoot)
}
$repositoryPrefix = $repositoryRoot.TrimEnd(
    [IO.Path]::DirectorySeparatorChar,
    [IO.Path]::AltDirectorySeparatorChar
) + [IO.Path]::DirectorySeparatorChar

# Git emits UTF-8; force its decoding only for this bounded inventory and restore the console.
$previousConsoleOutputEncoding = [Console]::OutputEncoding
try {
    [Console]::OutputEncoding = New-Object Text.UTF8Encoding($false)
    $relativePaths = @(
        & git -C $repositoryRoot -c core.quotepath=false `
            ls-files --cached --others --exclude-standard -- `
            ":(icase,glob)*.ps1" ":(icase,glob)**/*.ps1"
    )
} finally {
    [Console]::OutputEncoding = $previousConsoleOutputEncoding
}
if ($LASTEXITCODE -ne 0 -or $relativePaths.Count -le 0 -or
    $relativePaths.Count -gt $MaxPowerShellScripts) {
    throw "PowerShell syntax inventory failed."
}

$paths = foreach ($relativePath in $relativePaths) {
    if (
        [string]::IsNullOrWhiteSpace($relativePath) -or
        $relativePath.Length -gt 4096 -or
        [IO.Path]::IsPathRooted($relativePath) -or
        $relativePath -match "(^|[\\/])\.\.([\\/]|$)" -or
        [IO.Path]::GetExtension($relativePath) -ine ".ps1"
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

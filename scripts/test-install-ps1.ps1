[CmdletBinding()]
param(
    [switch]$ListOnly
)

$ErrorActionPreference = "Stop"
$MaxPowerShellScripts = 256
$repositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
. (Join-Path $PSScriptRoot "release\windows-powershell-source-validation.ps1")

$paths = @(Get-RepositoryPowerShellFiles $repositoryRoot $MaxPowerShellScripts)
if ($ListOnly) {
    $paths
    return
}

foreach ($path in $paths) {
    if ($null -ne (Get-PowerShellSourceFailure $path)) {
        Write-Error "PowerShell source validation failed."
        exit 1
    }
}

Write-Host "PowerShell syntax and source policy OK"

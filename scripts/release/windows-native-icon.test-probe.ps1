[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string]$IconPath,
    [Parameter(Mandatory)]
    [ValidateRange(1, 256)]
    [int]$Size
)

$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "windows-icon-validation.ps1")

$file = Get-BoundedPackageFile $IconPath 8388608
$icon = Get-FixedSizeNativeIcon $file $Size
try {
    [pscustomobject]@{
        PSVersion = $PSVersionTable.PSVersion.ToString()
        PSEdition = $PSVersionTable.PSEdition
        Width = $icon.Width
        Height = $icon.Height
        Hashes = @(Get-RenderedIconPixelHashes $icon)
    } | ConvertTo-Json -Compress
} finally {
    $icon.Dispose()
}

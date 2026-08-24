. (Join-Path $PSScriptRoot "windows-icon-validation.ps1")

function Get-BeaverExecutableBrandFailure(
    [string]$Path,
    [string]$ExpectedVersion,
    [string]$ExpectedIconPath = ""
) {
    $maxExecutableBytes = 536870912
    $maxIconBytes = 8388608
    if ($ExpectedVersion -notmatch "^[0-9]+\.[0-9]+\.[0-9]+$") {
        return "installed-brand-input"
    }

    try {
        $item = Get-BoundedPackageFile $Path $maxExecutableBytes
    } catch {
        return "installed-brand-input"
    }
    try {
        if ($item.VersionInfo.ProductName -cne "Beaver") {
            return "installed-brand-product"
        }
        if ($item.VersionInfo.FileVersion -cne $ExpectedVersion) {
            return "installed-brand-version"
        }
    } catch {
        return "installed-brand-input"
    }
    if ([string]::IsNullOrWhiteSpace($ExpectedIconPath)) {
        return $null
    }

    try {
        $iconItem = Get-BoundedPackageFile $ExpectedIconPath $maxIconBytes
    } catch {
        return "installed-brand-icon-reference"
    }
    $iconFailure = Get-NativeIconContentFailure $item $iconItem
    switch ($iconFailure) {
        { $_ -in @("reference-format", "reference-extract") } {
            return "installed-brand-icon-reference"
        }
        "actual-extract" { return "installed-brand-icon-extract" }
        "runtime" { return "installed-brand-icon-runtime" }
        "render" { return "installed-brand-icon-render" }
        "content" { return "installed-brand-icon-content" }
        $null { return $null }
        default { return "installed-brand-icon-runtime" }
    }
}

function Test-BeaverExecutableBrand(
    [string]$Path,
    [string]$ExpectedVersion,
    [string]$ExpectedIconPath = ""
) {
    return [string]::IsNullOrEmpty(
        (Get-BeaverExecutableBrandFailure $Path $ExpectedVersion $ExpectedIconPath)
    )
}

$MaxExtensionHostBytes = 4194304
$MaxNodeRuntimeBytes = 268435456
$MaxExtensionPowerShellScripts = 64
$MaxExtensionHostEntries = 20000

function Get-InstalledExtensionHostFailure([string]$InstallRoot) {
    $extensionHost = Join-ValidatedWindowsPath `
        $InstallRoot "resources\extension-host\host.mjs"
    $nodeRuntime = Join-ValidatedWindowsPath `
        $InstallRoot "resources\extension-host\runtime\node.exe"
    if (
        [string]::IsNullOrWhiteSpace($extensionHost) -or
        [string]::IsNullOrWhiteSpace($nodeRuntime) -or
        -not (Test-BoundedPackageFile $extensionHost $MaxExtensionHostBytes $InstallRoot) -or
        -not (Test-BoundedPackageFile $nodeRuntime $MaxNodeRuntimeBytes $InstallRoot)
    ) { return "binary" }

    # Generated npm wrappers are ignored by Git, so the installed tree is the authority here.
    $extensionHostRoot = Join-ValidatedWindowsPath $InstallRoot "resources\extension-host"
    if (
        [string]::IsNullOrWhiteSpace($extensionHostRoot) -or
        $null -ne (Get-PowerShellTreeFailure `
            $extensionHostRoot $MaxExtensionPowerShellScripts $MaxExtensionHostEntries)
    ) { return "source" }
    return $null
}

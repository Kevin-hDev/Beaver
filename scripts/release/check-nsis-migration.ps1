[CmdletBinding()]
param(
    [ValidateSet("Source", "Installed")]
    [string]$Mode = "Source",
    [string]$InstallerPath = ""
)

$ErrorActionPreference = "Stop"
$MaxSourceBytes = 65536
$MaxInstallerBytes = 2147483648
$MaxIconBytes = 8388608
$MaxUpdaterHelperBytes = 67108864
$MaxExtensionHostBytes = 4194304
$MaxNodeRuntimeBytes = 268435456
$Root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot "../.."))
$RootPrefix = $Root.TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar

function Stop-Validation {
    param(
        [Parameter(Mandatory)]
        [ValidateSet(
            "source-read", "source-config", "source-resource", "source-icons",
            "source-hook-required", "source-hook-forbidden", "source-installer",
            "source-installer-icon", "source-installer-icon-runtime",
            "installed-legacy-registry", "installed-registry",
            "installed-location", "installed-binary", "installed-brand-input",
            "installed-brand-product", "installed-brand-version", "installed-brand-icon-reference",
            "installed-brand-icon-extract", "installed-brand-icon-runtime",
            "installed-brand-icon-render", "installed-brand-icon-content",
            "installed-updater", "installed-extension-host", "installed-legacy-shortcuts",
            "installed-shortcuts"
        )]
        [string]$Code
    )
    # Fixed categories preserve generic failures while making CI regressions diagnosable.
    Write-Host "Windows package check failed: $Code"
    throw "Windows package validation failed."
}

. (Join-Path $PSScriptRoot "windows-artifact-helpers.ps1")

function Read-BoundedText([string]$RelativePath) {
    $path = [IO.Path]::GetFullPath((Join-Path $Root $RelativePath))
    if (-not $path.StartsWith($RootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        Stop-Validation "source-read"
    }
    $item = Get-Item -LiteralPath $path
    if (-not $item.PSIsContainer -and $item.Length -gt 0 -and $item.Length -le $MaxSourceBytes) {
        return [IO.File]::ReadAllText($item.FullName)
    }
    Stop-Validation "source-read"
}

function Test-SourceContracts {
    $config = Read-BoundedText "src-tauri/tauri.conf.json" | ConvertFrom-Json
    $windowsConfig = Read-BoundedText "src-tauri/tauri.windows.conf.json" | ConvertFrom-Json
    $helperResource = "target/updater-helper/cl-go-dash-updater.exe"
    if ($config.productName -ne "Beaver" -or $config.identifier -ne "com.clgo.dash") {
        Stop-Validation "source-config"
    }
    if ($config.bundle.windows.nsis.installerHooks -ne "windows/nsis-hooks.nsh") {
        Stop-Validation "source-config"
    }
    $helperProperty = $windowsConfig.bundle.resources.PSObject.Properties[$helperResource]
    if ($null -eq $helperProperty -or $helperProperty.Value -cne $helperResource) {
        Stop-Validation "source-resource"
    }
    $expectedIcons = @(
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
        "icons/icon.icns",
        "icons/icon.ico"
    )
    $actualIcons = @($config.bundle.icon)
    if ($actualIcons.Count -ne $expectedIcons.Count) {
        Stop-Validation "source-icons"
    }
    for ($index = 0; $index -lt $expectedIcons.Count; $index += 1) {
        if ($actualIcons[$index] -cne $expectedIcons[$index]) {
            Stop-Validation "source-icons"
        }
        $iconPath = Join-Path $Root "src-tauri/$($expectedIcons[$index])"
        if (-not (Test-BoundedPackageFile $iconPath $MaxIconBytes)) {
            Stop-Validation "source-icons"
        }
    }

    $hook = Read-BoundedText "src-tauri/windows/nsis-hooks.nsh"
    $required = @(
        "NSIS_HOOK_PREINSTALL",
        "NSIS_HOOK_POSTINSTALL",
        "Uninstall\CL-GO",
        "Uninstall\Beaver",
        "Software\clgo\CL-GO",
        "Software\clgo\Beaver",
        "IsShortcutTarget",
        "cl-go-dash.exe",
        "SetOutPath `$INSTDIR",
        "SHChangeNotify",
        "0x08000000"
    )
    foreach ($value in $required) {
        if (-not $hook.Contains($value)) {
            Stop-Validation "source-hook-required"
        }
    }

    $forbidden = @(
        ("Invoke-" + "Expression"),
        ("cmd" + ".exe"),
        ("Exec" + "Wait"),
        ("ns" + "Exec")
    )
    foreach ($value in $forbidden) {
        if ($hook.IndexOf($value, [StringComparison]::OrdinalIgnoreCase) -ge 0) {
            Stop-Validation "source-hook-forbidden"
        }
    }

    if ($InstallerPath) {
        $expected = "Beaver_{0}_x64-setup.exe" -f $config.version
        try {
            $installer = Get-BoundedPackageFile $InstallerPath $MaxInstallerBytes
        } catch {
            Stop-Validation "source-installer"
        }
        if ($installer.Name -cne $expected) {
            Stop-Validation "source-installer"
        }
        Test-AssociatedIcon $installer.FullName $MaxInstallerBytes
    }
}

function Get-ExistingRegistryPaths([string]$Suffix) {
    $roots = @(
        "Registry::HKEY_CURRENT_USER",
        "Registry::HKEY_LOCAL_MACHINE"
    )
    @($roots | ForEach-Object {
        $candidate = Join-Path $_ $Suffix
        if (Test-Path -LiteralPath $candidate) {
            $candidate
        }
    })
}

function Test-InstalledState {
    $oldUninstall = @(Get-ExistingRegistryPaths "Software\Microsoft\Windows\CurrentVersion\Uninstall\CL-GO")
    $newUninstall = @(Get-ExistingRegistryPaths "Software\Microsoft\Windows\CurrentVersion\Uninstall\Beaver")
    $oldProduct = @(Get-ExistingRegistryPaths "Software\clgo\CL-GO")
    $newProduct = @(Get-ExistingRegistryPaths "Software\clgo\Beaver")
    if ($oldUninstall.Count -ne 0 -or $oldProduct.Count -ne 0) {
        Stop-Validation "installed-legacy-registry"
    }
    if ($newUninstall.Count -ne 1 -or $newProduct.Count -ne 1) {
        Stop-Validation "installed-registry"
    }

    $metadata = Get-ItemProperty -LiteralPath $newUninstall[0]
    $installDir = [string]$metadata.InstallLocation
    $installDir = $installDir.Trim('"')
    if (-not (Test-FullyQualifiedWindowsPath $installDir)) {
        Stop-Validation "installed-location"
    }
    $binary = Join-ValidatedWindowsPath $installDir "cl-go-dash.exe"
    if ([string]::IsNullOrWhiteSpace($binary)) {
        Stop-Validation "installed-binary"
    }
    if (-not (Test-Path -LiteralPath $binary -PathType Leaf)) {
        Stop-Validation "installed-binary"
    }
    $expectedVersion = [string](Read-BoundedText "src-tauri/tauri.conf.json" | ConvertFrom-Json).version
    $expectedIcon = Join-Path $Root "src-tauri/icons/icon.ico"
    $brandFailure = Get-BeaverExecutableBrandFailure $binary $expectedVersion $expectedIcon
    if (-not [string]::IsNullOrEmpty($brandFailure)) {
        Stop-Validation $brandFailure
    }

    $helperPath = Join-ValidatedWindowsPath $installDir "target\updater-helper\cl-go-dash-updater.exe"
    if ([string]::IsNullOrWhiteSpace($helperPath)) {
        Stop-Validation "installed-updater"
    }
    if (-not (Test-BoundedPackageFile $helperPath $MaxUpdaterHelperBytes)) {
        Stop-Validation "installed-updater"
    }

    $extensionHost = Join-ValidatedWindowsPath $installDir "resources\extension-host\host.mjs"
    $nodeRuntime = Join-ValidatedWindowsPath $installDir "resources\extension-host\runtime\node.exe"
    if (
        -not (Test-BoundedPackageFile $extensionHost $MaxExtensionHostBytes) -or
        -not (Test-BoundedPackageFile $nodeRuntime $MaxNodeRuntimeBytes)
    ) {
        Stop-Validation "installed-extension-host"
    }

    $legacyShortcuts = @(
        (Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\CL-GO.lnk"),
        (Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\CL-GO\CL-GO.lnk"),
        (Join-Path $env:USERPROFILE "Desktop\CL-GO.lnk"),
        (Join-Path $env:ProgramData "Microsoft\Windows\Start Menu\Programs\CL-GO.lnk"),
        (Join-Path $env:PUBLIC "Desktop\CL-GO.lnk")
    )
    if ($legacyShortcuts.Where({ Test-Path -LiteralPath $_ }).Count -ne 0) {
        Stop-Validation "installed-legacy-shortcuts"
    }

    $startMenuShortcuts = @(
        (Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\Beaver.lnk"),
        (Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\Beaver\Beaver.lnk"),
        (Join-Path $env:ProgramData "Microsoft\Windows\Start Menu\Programs\Beaver.lnk"),
        (Join-Path $env:ProgramData "Microsoft\Windows\Start Menu\Programs\Beaver\Beaver.lnk")
    ).Where({ Test-Path -LiteralPath $_ })
    $desktopShortcuts = @(
        (Join-Path $env:USERPROFILE "Desktop\Beaver.lnk"),
        (Join-Path $env:PUBLIC "Desktop\Beaver.lnk")
    ).Where({ Test-Path -LiteralPath $_ })
    if (-not (Test-BeaverShortcutState $startMenuShortcuts $desktopShortcuts $binary)) {
        Stop-Validation "installed-shortcuts"
    }
}

Test-SourceContracts
if ($Mode -eq "Installed") {
    Test-InstalledState
}

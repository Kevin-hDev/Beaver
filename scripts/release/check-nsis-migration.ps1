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
    throw "Windows package validation failed."
}

. (Join-Path $PSScriptRoot "windows-artifact-helpers.ps1")

function Read-BoundedText([string]$RelativePath) {
    $path = [IO.Path]::GetFullPath((Join-Path $Root $RelativePath))
    if (-not $path.StartsWith($RootPrefix, [StringComparison]::OrdinalIgnoreCase)) {
        Stop-Validation
    }
    $item = Get-Item -LiteralPath $path
    if (-not $item.PSIsContainer -and $item.Length -gt 0 -and $item.Length -le $MaxSourceBytes) {
        return [IO.File]::ReadAllText($item.FullName)
    }
    Stop-Validation
}

function Test-SourceContracts {
    $config = Read-BoundedText "src-tauri/tauri.conf.json" | ConvertFrom-Json
    $windowsConfig = Read-BoundedText "src-tauri/tauri.windows.conf.json" | ConvertFrom-Json
    $helperResource = "target/updater-helper/cl-go-dash-updater.exe"
    if ($config.productName -ne "Beaver" -or $config.identifier -ne "com.clgo.dash") {
        Stop-Validation
    }
    if ($config.bundle.windows.nsis.installerHooks -ne "windows/nsis-hooks.nsh") {
        Stop-Validation
    }
    $helperProperty = $windowsConfig.bundle.resources.PSObject.Properties[$helperResource]
    if ($null -eq $helperProperty -or $helperProperty.Value -cne $helperResource) {
        Stop-Validation
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
        Stop-Validation
    }
    for ($index = 0; $index -lt $expectedIcons.Count; $index += 1) {
        if ($actualIcons[$index] -cne $expectedIcons[$index]) {
            Stop-Validation
        }
        $icon = Get-Item -LiteralPath (Join-Path $Root "src-tauri/$($expectedIcons[$index])")
        if ($icon.PSIsContainer -or $icon.Length -le 0 -or $icon.Length -gt $MaxIconBytes) {
            Stop-Validation
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
            Stop-Validation
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
            Stop-Validation
        }
    }

    if ($InstallerPath) {
        $installer = Get-Item -LiteralPath $InstallerPath
        $expected = "Beaver_{0}_x64-setup.exe" -f $config.version
        $isLink = ($installer.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0
        if (
            $installer.PSIsContainer -or
            $isLink -or
            $installer.Length -le 0 -or
            $installer.Length -gt $MaxInstallerBytes -or
            $installer.Name -cne $expected
        ) {
            Stop-Validation
        }
        Test-AssociatedIcon $installer.FullName
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
        Stop-Validation
    }
    if ($newUninstall.Count -ne 1 -or $newProduct.Count -ne 1) {
        Stop-Validation
    }

    $metadata = Get-ItemProperty -LiteralPath $newUninstall[0]
    $installDir = [string]$metadata.InstallLocation
    $installDir = $installDir.Trim('"')
    if (-not (Test-FullyQualifiedWindowsPath $installDir)) {
        Stop-Validation
    }
    $binary = Join-ValidatedWindowsPath $installDir "cl-go-dash.exe"
    if ([string]::IsNullOrWhiteSpace($binary)) {
        Stop-Validation
    }
    if (-not (Test-Path -LiteralPath $binary -PathType Leaf)) {
        Stop-Validation
    }
    $expectedVersion = [string](Read-BoundedText "src-tauri/tauri.conf.json" | ConvertFrom-Json).version
    $expectedIcon = Join-Path $Root "src-tauri/icons/icon.ico"
    if (-not (Test-BeaverExecutableBrand $binary $expectedVersion $expectedIcon)) {
        Stop-Validation
    }

    $helperPath = Join-ValidatedWindowsPath $installDir "target\updater-helper\cl-go-dash-updater.exe"
    if ([string]::IsNullOrWhiteSpace($helperPath)) {
        Stop-Validation
    }
    if (-not (Test-UpdaterHelper $helperPath $MaxUpdaterHelperBytes)) {
        Stop-Validation
    }

    $extensionHost = Join-ValidatedWindowsPath $installDir "resources\extension-host\host.mjs"
    $nodeRuntime = Join-ValidatedWindowsPath $installDir "resources\extension-host\runtime\node.exe"
    if (
        -not (Test-UpdaterHelper $extensionHost $MaxExtensionHostBytes) -or
        -not (Test-UpdaterHelper $nodeRuntime $MaxNodeRuntimeBytes)
    ) {
        Stop-Validation
    }

    $legacyShortcuts = @(
        (Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\CL-GO.lnk"),
        (Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\CL-GO\CL-GO.lnk"),
        (Join-Path $env:USERPROFILE "Desktop\CL-GO.lnk"),
        (Join-Path $env:ProgramData "Microsoft\Windows\Start Menu\Programs\CL-GO.lnk"),
        (Join-Path $env:PUBLIC "Desktop\CL-GO.lnk")
    )
    if ($legacyShortcuts.Where({ Test-Path -LiteralPath $_ }).Count -ne 0) {
        Stop-Validation
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
        Stop-Validation
    }
}

Test-SourceContracts
if ($Mode -eq "Installed") {
    Test-InstalledState
}

$ErrorActionPreference = "Stop"

function Assert-True([bool]$Value) {
    if (-not $Value) { throw "PowerShell package contract failed." }
}

function Assert-False([bool]$Value) {
    if ($Value) { throw "PowerShell package contract failed." }
}

function Stop-Validation([string]$Code) {
    throw "Windows package check failed: $Code"
}

. (Join-Path $PSScriptRoot "windows-icon-validation.test.ps1")
. (Join-Path $PSScriptRoot "windows-native-icon-engine.test.ps1")
. (Join-Path $PSScriptRoot "windows-package-file.test.ps1")
. (Join-Path $PSScriptRoot "windows-powershell-syntax-inventory.test.ps1")

function New-TestShortcut([string]$Path, [string]$Target) {
    $shell = New-Object -ComObject WScript.Shell
    try {
        $shortcut = $shell.CreateShortcut($Path)
        $shortcut.TargetPath = $Target
        $shortcut.Save()
    } finally {
        [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($shell)
    }
}

. (Join-Path $PSScriptRoot "windows-artifact-helpers.ps1")

$randomBytes = New-Object byte[] 16
$random = [Security.Cryptography.RandomNumberGenerator]::Create()
try {
    $random.GetBytes($randomBytes)
} finally {
    $random.Dispose()
}
$directoryName = -join @($randomBytes | ForEach-Object { $_.ToString("x2") })
$temporaryBase = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd("\")
$temporaryRoot = [IO.Path]::GetFullPath((Join-Path $temporaryBase $directoryName))

try {
    [void](New-Item -ItemType Directory -Path $temporaryRoot)
    $binary = Join-Path $temporaryRoot "cl-go-dash.exe"
    [IO.File]::WriteAllBytes($binary, [byte[]]@(1))
    $startLink = Join-Path $temporaryRoot "start.lnk"
    $desktopLink = Join-Path $temporaryRoot "desktop.lnk"
    $secondDesktop = Join-Path $temporaryRoot "desktop-second.lnk"
    New-TestShortcut $startLink $binary
    New-TestShortcut $desktopLink $binary
    New-TestShortcut $secondDesktop $binary

    Assert-True (Test-FullyQualifiedWindowsPath "C:\Apps\Beaver")
    Assert-False (Test-FullyQualifiedWindowsPath "C:Beaver")
    Assert-False (Test-FullyQualifiedWindowsPath "C:\Apps\..\Beaver")
    Assert-True ($null -ne (Get-Command Join-ValidatedWindowsPath -ErrorAction SilentlyContinue))
    $extendedBinary = Join-ValidatedWindowsPath "\\?\C:\Apps\Beaver" "cl-go-dash.exe"
    Assert-True ($extendedBinary -ceq "C:\Apps\Beaver\cl-go-dash.exe")
    Assert-True ($null -eq (Join-ValidatedWindowsPath "C:\Apps\Beaver" "..\outside.exe"))
    Assert-True ($null -eq (Join-ValidatedWindowsPath "C:\Apps\Beaver" "sub\..\inside.exe"))
    Assert-True ($null -eq (Join-ValidatedWindowsPath "C:\Apps\Beaver" "sub/../inside.exe"))
    Assert-True ($null -eq (Join-ValidatedWindowsPath "C:\Apps\Beaver" "D:\outside.exe"))
    Assert-True (Test-BeaverShortcutState @($startLink) @() $binary)
    Assert-True (Test-BeaverShortcutState @($startLink) @($desktopLink) $binary)
    Assert-False (Test-BeaverShortcutState @($startLink) @($desktopLink, $secondDesktop) $binary)
    Assert-False (Test-BeaverShortcutState @() @() $binary)

    $validHelper = Join-Path $temporaryRoot "valid-updater.exe"
    $emptyHelper = Join-Path $temporaryRoot "empty-updater.exe"
    [IO.File]::WriteAllBytes($validHelper, [byte[]]@(1, 2, 3))
    [IO.File]::WriteAllBytes($emptyHelper, [byte[]]@())
    Assert-True (Test-UpdaterHelper $validHelper 67108864)
    Assert-False (Test-UpdaterHelper $emptyHelper 67108864)
    Assert-False (Test-BeaverExecutableBrand $validHelper "1.1.1" $validHelper)
    Assert-True ((Get-BeaverExecutableBrandFailure $validHelper "1.1.1" $validHelper) -ceq "installed-brand-product")
    Assert-True ((Get-BeaverExecutableBrandFailure "" "1.1.1" $validHelper) -ceq "installed-brand-input")

    $referenceIconPath = [IO.Path]::GetFullPath(
        (Join-Path $PSScriptRoot "../../src-tauri/icons/icon.ico")
    )
    $compiler = @(
        (Join-Path $env:WINDIR "Microsoft.NET\Framework64\v4.0.30319\csc.exe"),
        (Join-Path $env:WINDIR "Microsoft.NET\Framework\v4.0.30319\csc.exe")
    ).Where({ Test-Path -LiteralPath $_ -PathType Leaf }, "First")
    Assert-True ($compiler.Count -eq 1)
    $fixtureSource = Join-Path $temporaryRoot "plain-fixture.cs"
    $plainExecutable = Join-Path $temporaryRoot "plain.exe"
    [IO.File]::WriteAllText(
        $fixtureSource,
        "public static class BeaverPlainFixture { public static void Main() {} }"
    )
    & $compiler[0] /nologo /target:winexe "/out:$plainExecutable" $fixtureSource
    Assert-True ($LASTEXITCODE -eq 0)
    Assert-True (
        (Get-NativeIconResourceFailure $plainExecutable 67108864) -ceq "extract"
    )
    $plainRejected = $false
    try {
        Test-AssociatedIcon $plainExecutable 67108864
    } catch {
        $plainRejected = $_.Exception.Message -ceq `
            "Windows package check failed: source-installer-icon"
    }
    Assert-True $plainRejected

    $originalInteropIdentity = $script:NativeIconInteropType.FullName
    $modifiedModuleRoot = Join-Path $temporaryRoot "modified-native-module"
    [void](New-Item -ItemType Directory -Path $modifiedModuleRoot)
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot "windows-package-file.ps1") `
        -Destination $modifiedModuleRoot
    $nativeModulePath = Join-Path $PSScriptRoot "windows-native-icon.ps1"
    $modifiedNativeModulePath = Join-Path $modifiedModuleRoot "windows-native-icon.ps1"
    $nativeModuleSource = [IO.File]::ReadAllText($nativeModulePath)
    $modifiedNativeModuleSource = $nativeModuleSource.Replace(
        "return DestroyIcon(iconHandle);",
        "DestroyIcon(iconHandle); return false;"
    )
    Assert-False ($modifiedNativeModuleSource -ceq $nativeModuleSource)
    [IO.File]::WriteAllText($modifiedNativeModulePath, $modifiedNativeModuleSource)
    . $modifiedNativeModulePath
    Assert-False ($script:NativeIconInteropType.FullName -ceq $originalInteropIdentity)
    Assert-True (
        (Get-NativeIconResourceFailure $referenceIconPath 8388608) -ceq "runtime"
    )
    . $nativeModulePath
    Assert-True ($script:NativeIconInteropType.FullName -ceq $originalInteropIdentity)

    $transparentRed = New-Object Drawing.Bitmap 2, 2
    $transparentBlue = New-Object Drawing.Bitmap 2, 2
    $visibleRed = New-Object Drawing.Bitmap 2, 2
    try {
        $transparentRed.SetPixel(0, 0, [Drawing.Color]::FromArgb(0, 255, 0, 0))
        $transparentBlue.SetPixel(0, 0, [Drawing.Color]::FromArgb(0, 0, 0, 255))
        $visibleRed.SetPixel(0, 0, [Drawing.Color]::FromArgb(255, 255, 0, 0))
        foreach ($background in @([Drawing.Color]::Black, [Drawing.Color]::White)) {
            $backgroundArgb = $background.ToArgb()
            $firstHash = Get-VisibleBitmapPixelHash $transparentRed $backgroundArgb
            $secondHash = Get-VisibleBitmapPixelHash $transparentBlue $backgroundArgb
            $visibleHash = Get-VisibleBitmapPixelHash $visibleRed $backgroundArgb
            Assert-True ($firstHash -ceq $secondHash)
            Assert-False ($firstHash -ceq $visibleHash)
        }
    } finally {
        $transparentRed.Dispose()
        $transparentBlue.Dispose()
        $visibleRed.Dispose()
    }
} finally {
    if (Test-Path -LiteralPath $temporaryRoot) {
        if (
            [IO.Path]::GetDirectoryName($temporaryRoot) -cne $temporaryBase -or
            [IO.Path]::GetFileName($temporaryRoot) -notmatch "^[a-f0-9]{32}$"
        ) {
            throw "PowerShell package contract failed."
        }
        Remove-Item -LiteralPath $temporaryRoot -Recurse -Force
    }
}

& (Join-Path $PSScriptRoot "../test-install-ps1.ps1")
& (Join-Path $PSScriptRoot "check-nsis-migration.ps1") -Mode Source
Write-Host "PowerShell package contracts OK"

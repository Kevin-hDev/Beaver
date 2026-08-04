$ErrorActionPreference = "Stop"

function Assert-True([bool]$Value) {
    if (-not $Value) { throw "PowerShell package contract failed." }
}

function Assert-False([bool]$Value) {
    if ($Value) { throw "PowerShell package contract failed." }
}

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

Write-Host "PowerShell package contracts OK"

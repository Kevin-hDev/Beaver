$ErrorActionPreference = "Stop"

function Assert-True([bool]$Value) {
    if (-not $Value) { throw "Data fingerprint test failed." }
}

function Assert-Equal($Expected, $Actual) {
    if ($Expected -cne $Actual) { throw "Data fingerprint test failed." }
}

function Assert-Throws([scriptblock]$Action) {
    try {
        & $Action
    } catch {
        if ($_.Exception.Message -cne "Data validation failed.") {
            throw "Data fingerprint test failed."
        }
        return
    }
    throw "Data fingerprint test failed."
}

$randomBytes = New-Object byte[] 16
$random = [Security.Cryptography.RandomNumberGenerator]::Create()
try {
    $random.GetBytes($randomBytes)
} finally {
    $random.Dispose()
}
$name = -join @($randomBytes | ForEach-Object { $_.ToString("x2") })
$temporaryBase = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd("\")
$temporaryRoot = [IO.Path]::GetFullPath((Join-Path $temporaryBase $name))
$source = Join-Path $temporaryRoot "source"
$backup = Join-Path $temporaryRoot "backup"
$script = Join-Path $PSScriptRoot "windows-data-fingerprint.ps1"

try {
    [void](New-Item -ItemType Directory -Path (Join-Path $source "a") -Force)
    [void](New-Item -ItemType Directory -Path (Join-Path $source "b") -Force)
    [IO.File]::WriteAllText((Join-Path $source "a\one.txt"), "one")
    [IO.File]::WriteAllText((Join-Path $source "b\two.txt"), "two")

    $first = & $script -Source $source -BackupDirectory $backup
    $second = & $script -Source $source
    Assert-Equal $first $second
    Assert-True ($first -match "^[A-F0-9]{64}$")
    Assert-True (Test-Path -LiteralPath (Join-Path $backup "a\one.txt") -PathType Leaf)
    Assert-Equal "one" ([IO.File]::ReadAllText((Join-Path $backup "a\one.txt")))
    Assert-Throws { & $script -Source $source -MaxFiles 1 }
    Assert-Throws { & $script -Source $source -MaxBytes 1 }

    $link = Join-Path $source "linked.txt"
    try {
        [void](New-Item -ItemType SymbolicLink -Path $link -Target (Join-Path $source "a\one.txt"))
        Assert-Throws { & $script -Source $source }
    } catch {
        if ($_.Exception.Message -eq "Data fingerprint test failed.") { throw }
    }
} finally {
    if (Test-Path -LiteralPath $temporaryRoot) {
        if (
            [IO.Path]::GetDirectoryName($temporaryRoot) -cne $temporaryBase -or
            [IO.Path]::GetFileName($temporaryRoot) -notmatch "^[a-f0-9]{32}$"
        ) {
            throw "Data fingerprint test failed."
        }
        Remove-Item -LiteralPath $temporaryRoot -Recurse -Force
    }
}

Write-Host "Windows data fingerprint contracts OK"

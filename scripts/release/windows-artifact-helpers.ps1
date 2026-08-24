# Branding has its own bounded failure contract so Windows handoffs identify the exact stage.
. (Join-Path $PSScriptRoot "windows-brand-validation.ps1")

function Test-AssociatedIcon([string]$Path, [long]$MaxBytes) {
    $failure = Get-NativeIconResourceFailure $Path $MaxBytes
    if ($failure -ceq "runtime") {
        Stop-Validation "source-installer-icon-runtime"
    }
    if ($null -ne $failure) {
        Stop-Validation "source-installer-icon"
    }
}

function Test-ShortcutTarget([string]$Path, [string]$ExpectedTarget) {
    $shell = New-Object -ComObject WScript.Shell
    try {
        $target = $shell.CreateShortcut($Path).TargetPath
        return $target.Equals($ExpectedTarget, [StringComparison]::OrdinalIgnoreCase)
    } finally {
        [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($shell)
    }
}

function Test-FullyQualifiedWindowsPath([string]$Path) {
    if ([string]::IsNullOrWhiteSpace($Path) -or $Path.Contains("..")) {
        return $false
    }
    try {
        $root = [IO.Path]::GetPathRoot($Path)
        [void][IO.Path]::GetFullPath($Path)
    } catch {
        return $false
    }
    return -not [string]::IsNullOrEmpty($root) -and $root.EndsWith("\")
}

function Join-ValidatedWindowsPath([string]$BasePath, [string]$ChildPath) {
    if (
        -not (Test-FullyQualifiedWindowsPath $BasePath) -or
        [string]::IsNullOrWhiteSpace($ChildPath) -or
        [IO.Path]::IsPathRooted($ChildPath) -or
        $ChildPath -match "(^|[\\/])\.\.([\\/]|$)"
    ) {
        return $null
    }

    if ($BasePath.StartsWith("\\?\UNC\", [StringComparison]::OrdinalIgnoreCase)) {
        $normalizedBase = "\\" + $BasePath.Substring(8)
    } elseif ($BasePath.StartsWith("\\?\", [StringComparison]::Ordinal)) {
        $normalizedBase = $BasePath.Substring(4)
        if ($normalizedBase -notmatch "^[A-Za-z]:\\") {
            return $null
        }
    } else {
        $normalizedBase = $BasePath
    }

    try {
        $base = [IO.Path]::GetFullPath($normalizedBase).TrimEnd("\")
        $joined = [IO.Path]::GetFullPath([IO.Path]::Combine($base, $ChildPath))
        $prefix = $base + "\"
        if (-not $joined.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
            return $null
        }
        return $joined
    } catch {
        return $null
    }
}

function Test-BeaverShortcutState(
    [object[]]$StartMenuShortcuts,
    [object[]]$DesktopShortcuts,
    [string]$ExpectedTarget
) {
    if (
        [string]::IsNullOrWhiteSpace($ExpectedTarget) -or
        $StartMenuShortcuts.Count -ne 1 -or
        $DesktopShortcuts.Count -gt 1
    ) {
        return $false
    }
    try {
        if (-not (Test-ShortcutTarget $StartMenuShortcuts[0] $ExpectedTarget)) {
            return $false
        }
        return $DesktopShortcuts.Count -eq 0 -or
            (Test-ShortcutTarget $DesktopShortcuts[0] $ExpectedTarget)
    } catch {
        return $false
    }
}

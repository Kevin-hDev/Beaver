function Test-AssociatedIcon([string]$Path) {
    Add-Type -AssemblyName System.Drawing
    $icon = [Drawing.Icon]::ExtractAssociatedIcon($Path)
    if ($null -eq $icon) {
        Stop-Validation "source-installer-icon"
    }
    try {
        if ($icon.Width -le 0 -or $icon.Height -le 0) {
            Stop-Validation "source-installer-icon"
        }
    } finally {
        $icon.Dispose()
    }
}

function Get-VisibleBitmapPixelHash([Drawing.Bitmap]$Source, [int]$BackgroundArgb) {
    if ($null -eq $Source) {
        return $null
    }

    if ($Source.Width -le 0 -or $Source.Height -le 0 -or $Source.Width -gt 256 -or $Source.Height -gt 256) {
        return $null
    }

    $bitmap = New-Object Drawing.Bitmap $Source.Width, $Source.Height
    try {
        $graphics = [Drawing.Graphics]::FromImage($bitmap)
        try {
            $graphics.Clear([Drawing.Color]::FromArgb($BackgroundArgb))
            $graphics.DrawImageUnscaled($Source, 0, 0)
        } finally {
            $graphics.Dispose()
        }
        $pixels = New-Object byte[] ($bitmap.Width * $bitmap.Height * 4)
        $offset = 0
        for ($y = 0; $y -lt $bitmap.Height; $y += 1) {
            for ($x = 0; $x -lt $bitmap.Width; $x += 1) {
                [BitConverter]::GetBytes($bitmap.GetPixel($x, $y).ToArgb()).CopyTo($pixels, $offset)
                $offset += 4
            }
        }
        $sha256 = [Security.Cryptography.SHA256]::Create()
        try {
            return -join @($sha256.ComputeHash($pixels) | ForEach-Object { $_.ToString("x2") })
        } finally {
            $sha256.Dispose()
        }
    } finally {
        $bitmap.Dispose()
    }
}

function Get-RenderedIconPixelHashes([Drawing.Icon]$Icon) {
    if ($null -eq $Icon) {
        return @()
    }

    $source = $Icon.ToBitmap()
    try {
        # Two opaque backgrounds compare only pixels Windows can actually display.
        return @(
            (Get-VisibleBitmapPixelHash $source ([Drawing.Color]::Black.ToArgb())),
            (Get-VisibleBitmapPixelHash $source ([Drawing.Color]::White.ToArgb()))
        )
    } finally {
        $source.Dispose()
    }
}

function Test-BeaverExecutableBrand(
    [string]$Path,
    [string]$ExpectedVersion,
    [string]$ExpectedIconPath
) {
    $maxExecutableBytes = 536870912
    $maxIconBytes = 8388608
    if (
        [string]::IsNullOrWhiteSpace($Path) -or
        [string]::IsNullOrWhiteSpace($ExpectedIconPath) -or
        $ExpectedVersion -notmatch "^[0-9]+\.[0-9]+\.[0-9]+$" -or
        -not (Test-Path -LiteralPath $Path -PathType Leaf) -or
        -not (Test-Path -LiteralPath $ExpectedIconPath -PathType Leaf)
    ) {
        return $false
    }

    try {
        $item = Get-Item -LiteralPath $Path -Force
        $iconItem = Get-Item -LiteralPath $ExpectedIconPath -Force
        $isLink = ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0
        $iconIsLink = ($iconItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0
        if (
            $isLink -or $item.Length -le 0 -or $item.Length -gt $maxExecutableBytes -or
            $iconIsLink -or $iconItem.Length -le 0 -or $iconItem.Length -gt $maxIconBytes
        ) {
            return $false
        }
        if (
            $item.VersionInfo.ProductName -cne "Beaver" -or
            $item.VersionInfo.FileVersion -cne $ExpectedVersion
        ) {
            return $false
        }

        Add-Type -AssemblyName System.Drawing
        # The packaged ICO is authoritative, so executable validation follows branding changes automatically.
        $expectedIcon = [Drawing.Icon]::new($iconItem.FullName, 32, 32)
        try {
            $actualIcon = [Drawing.Icon]::ExtractAssociatedIcon($item.FullName)
            if ($null -eq $actualIcon) {
                return $false
            }
            try {
                $actualHashes = @(Get-RenderedIconPixelHashes $actualIcon)
                $expectedHashes = @(Get-RenderedIconPixelHashes $expectedIcon)
                return $actualHashes.Count -eq 2 -and
                    $expectedHashes.Count -eq 2 -and
                    -not [string]::IsNullOrWhiteSpace($actualHashes[0]) -and
                    -not [string]::IsNullOrWhiteSpace($actualHashes[1]) -and
                    $actualHashes[0] -ceq $expectedHashes[0] -and
                    $actualHashes[1] -ceq $expectedHashes[1]
            } finally {
                $actualIcon.Dispose()
            }
        } finally {
            $expectedIcon.Dispose()
        }
    } catch {
        return $false
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

function Test-UpdaterHelper([string]$Path, [long]$MaxBytes) {
    if (
        [string]::IsNullOrWhiteSpace($Path) -or
        $MaxBytes -le 0 -or
        -not (Test-Path -LiteralPath $Path -PathType Leaf)
    ) {
        return $false
    }
    try {
        $item = Get-Item -LiteralPath $Path -Force
        $isLink = ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0
        return -not $item.PSIsContainer -and -not $isLink -and
            $item.Length -gt 0 -and $item.Length -le $MaxBytes
    } catch {
        return $false
    }
}

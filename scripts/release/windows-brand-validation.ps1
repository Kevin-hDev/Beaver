. (Join-Path $PSScriptRoot "windows-native-icon.ps1")

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

function Get-BeaverExecutableBrandFailure(
    [string]$Path,
    [string]$ExpectedVersion,
    [string]$ExpectedIconPath = ""
) {
    $maxExecutableBytes = 536870912
    $maxIconBytes = 8388608
    if (
        [string]::IsNullOrWhiteSpace($Path) -or
        $ExpectedVersion -notmatch "^[0-9]+\.[0-9]+\.[0-9]+$" -or
        -not (Test-Path -LiteralPath $Path -PathType Leaf)
    ) {
        return "installed-brand-input"
    }

    try {
        $item = Get-Item -LiteralPath $Path -Force
        $isLink = ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0
        if ($isLink -or $item.Length -le 0 -or $item.Length -gt $maxExecutableBytes) {
            return "installed-brand-input"
        }
        if ($item.VersionInfo.ProductName -cne "Beaver") {
            return "installed-brand-product"
        }
        if ($item.VersionInfo.FileVersion -cne $ExpectedVersion) {
            return "installed-brand-version"
        }
        if ([string]::IsNullOrWhiteSpace($ExpectedIconPath)) {
            return $null
        }
        if (-not (Test-Path -LiteralPath $ExpectedIconPath -PathType Leaf)) {
            return "installed-brand-icon-reference"
        }
        $iconItem = Get-Item -LiteralPath $ExpectedIconPath -Force
        $iconIsLink = ($iconItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0
        if ($iconIsLink -or $iconItem.Length -le 0 -or $iconItem.Length -gt $maxIconBytes) {
            return "installed-brand-icon-reference"
        }

        Add-Type -AssemblyName System.Drawing
        try {
            $expectedIcon = Get-FixedSizeNativeIcon $iconItem.FullName 32
        } catch {
            return "installed-brand-icon-reference"
        }
        try {
            try {
                $actualIcon = Get-FixedSizeNativeIcon $item.FullName 32
            } catch {
                return "installed-brand-icon-extract"
            }
            if ($null -eq $actualIcon) {
                return "installed-brand-icon-extract"
            }
            try {
                if ($actualIcon.Width -ne $expectedIcon.Width -or $actualIcon.Height -ne $expectedIcon.Height) {
                    return "installed-brand-icon-size"
                }
                try {
                    $actualHashes = @(Get-RenderedIconPixelHashes $actualIcon)
                    $expectedHashes = @(Get-RenderedIconPixelHashes $expectedIcon)
                } catch {
                    return "installed-brand-icon-render"
                }
                if (
                    $actualHashes.Count -ne 2 -or
                    $expectedHashes.Count -ne 2 -or
                    [string]::IsNullOrWhiteSpace($actualHashes[0]) -or
                    [string]::IsNullOrWhiteSpace($actualHashes[1])
                ) {
                    return "installed-brand-icon-render"
                }
                if (
                    $actualHashes[0] -cne $expectedHashes[0] -or
                    $actualHashes[1] -cne $expectedHashes[1]
                ) {
                    return "installed-brand-icon-content"
                }
                return $null
            } finally {
                $actualIcon.Dispose()
            }
        } finally {
            $expectedIcon.Dispose()
        }
    } catch {
        return "installed-brand-input"
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

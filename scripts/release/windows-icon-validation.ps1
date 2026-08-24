. (Join-Path $PSScriptRoot "windows-native-icon.ps1")

$script:WindowsBrandIconSizes = @(32, 48, 256)

function Get-WindowsIcoFrameSizes([IO.FileInfo]$File) {
    if ($null -eq $File -or $File.Extension -ine ".ico") {
        throw (New-Object IO.InvalidDataException("Invalid icon reference."))
    }
    $bytes = [IO.File]::ReadAllBytes($File.FullName)
    if (
        $bytes.Length -lt 22 -or
        [BitConverter]::ToUInt16($bytes, 0) -ne 0 -or
        [BitConverter]::ToUInt16($bytes, 2) -ne 1
    ) {
        throw (New-Object IO.InvalidDataException("Invalid icon reference."))
    }
    $count = [int][BitConverter]::ToUInt16($bytes, 4)
    $directoryEnd = 6 + (16 * $count)
    if ($count -le 0 -or $count -gt 256 -or $directoryEnd -gt $bytes.Length) {
        throw (New-Object IO.InvalidDataException("Invalid icon reference."))
    }

    $sizes = New-Object int[] $count
    for ($index = 0; $index -lt $count; $index += 1) {
        $offset = 6 + (16 * $index)
        $width = [int]$bytes[$offset]
        $height = [int]$bytes[$offset + 1]
        if ($width -eq 0) { $width = 256 }
        if ($height -eq 0) { $height = 256 }
        $imageBytes = [uint64][BitConverter]::ToUInt32($bytes, $offset + 8)
        $imageOffset = [uint64][BitConverter]::ToUInt32($bytes, $offset + 12)
        $imageEnd = $imageOffset + $imageBytes
        if (
            $width -ne $height -or
            $imageBytes -eq 0 -or
            $imageOffset -lt $directoryEnd -or
            $imageEnd -gt $bytes.Length
        ) {
            throw (New-Object IO.InvalidDataException("Invalid icon reference."))
        }
        $sizes[$index] = $width
    }
    return $sizes
}

function Test-WindowsReferenceIcon([IO.FileInfo]$File) {
    try {
        $sizes = @(Get-WindowsIcoFrameSizes $File)
        foreach ($requiredSize in $script:WindowsBrandIconSizes) {
            if ($requiredSize -notin $sizes) {
                return $false
            }
        }
        return $true
    } catch {
        return $false
    }
}

function Get-VisibleBitmapPixelHash([Drawing.Bitmap]$Source, [int]$BackgroundArgb) {
    if ($null -eq $Source -or $Source.Width -le 0 -or $Source.Height -le 0) {
        return $null
    }
    if ($Source.Width -gt 256 -or $Source.Height -gt 256) {
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
        $pixelOffset = 0
        for ($y = 0; $y -lt $bitmap.Height; $y += 1) {
            for ($x = 0; $x -lt $bitmap.Width; $x += 1) {
                [BitConverter]::GetBytes($bitmap.GetPixel($x, $y).ToArgb()).CopyTo(
                    $pixels,
                    $pixelOffset
                )
                $pixelOffset += 4
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
        # Two backgrounds compare only pixels Windows can display, including transparency.
        return @(
            (Get-VisibleBitmapPixelHash $source ([Drawing.Color]::Black.ToArgb())),
            (Get-VisibleBitmapPixelHash $source ([Drawing.Color]::White.ToArgb()))
        )
    } finally {
        $source.Dispose()
    }
}

function Get-NativeIconContentFailure([IO.FileInfo]$ActualFile, [IO.FileInfo]$ReferenceFile) {
    if (-not (Test-WindowsReferenceIcon $ReferenceFile)) {
        return "reference-format"
    }

    foreach ($size in $script:WindowsBrandIconSizes) {
        $expectedIcon = $null
        $actualIcon = $null
        try {
            try {
                $expectedIcon = Get-FixedSizeNativeIcon $ReferenceFile $size
            } catch [Runtime.InteropServices.ExternalException] {
                return "runtime"
            } catch {
                return "reference-extract"
            }
            try {
                $actualIcon = Get-FixedSizeNativeIcon $ActualFile $size
            } catch [Runtime.InteropServices.ExternalException] {
                return "runtime"
            } catch {
                return "actual-extract"
            }
            try {
                $actualHashes = @(Get-RenderedIconPixelHashes $actualIcon)
                $expectedHashes = @(Get-RenderedIconPixelHashes $expectedIcon)
            } catch {
                return "render"
            }
            if (
                $actualHashes.Count -ne 2 -or
                $expectedHashes.Count -ne 2 -or
                [string]::IsNullOrWhiteSpace($actualHashes[0]) -or
                [string]::IsNullOrWhiteSpace($actualHashes[1]) -or
                [string]::IsNullOrWhiteSpace($expectedHashes[0]) -or
                [string]::IsNullOrWhiteSpace($expectedHashes[1])
            ) {
                return "render"
            }
            if (
                $actualHashes[0] -cne $expectedHashes[0] -or
                $actualHashes[1] -cne $expectedHashes[1]
            ) {
                return "content"
            }
        } finally {
            if ($null -ne $actualIcon) { $actualIcon.Dispose() }
            if ($null -ne $expectedIcon) { $expectedIcon.Dispose() }
        }
    }
    return $null
}

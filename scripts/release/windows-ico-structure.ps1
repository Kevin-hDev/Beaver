function Get-BigEndianUInt32([byte[]]$Bytes, [int]$Offset) {
    return [uint32](
        ([uint32]$Bytes[$Offset] * 16777216) +
        ([uint32]$Bytes[$Offset + 1] * 65536) +
        ([uint32]$Bytes[$Offset + 2] * 256) +
        [uint32]$Bytes[$Offset + 3]
    )
}

function Test-ContiguousDibMask([uint32]$Mask) {
    if ($Mask -eq 0) { return $false }
    $value = [uint64]$Mask
    while (($value -band 1) -eq 0) { $value = $value -shr 1 }
    while (($value -band 1) -eq 1) { $value = $value -shr 1 }
    return $value -eq 0
}

function Get-IcoPayloadDimensions([byte[]]$Bytes, [int]$Offset, [int]$Length) {
    $pngSignature = [byte[]]@(137, 80, 78, 71, 13, 10, 26, 10)
    $isPng = $Length -ge 33
    for ($index = 0; $isPng -and $index -lt $pngSignature.Length; $index += 1) {
        $isPng = $Bytes[$Offset + $index] -eq $pngSignature[$index]
    }
    if ($isPng) {
        $chunkLength = Get-BigEndianUInt32 $Bytes ($Offset + 8)
        $chunkType = [Text.Encoding]::ASCII.GetString($Bytes, $Offset + 12, 4)
        if ($chunkLength -ne 13 -or $chunkType -cne "IHDR") {
            throw (New-Object IO.InvalidDataException("Invalid icon reference."))
        }
        return [pscustomobject]@{
            Width = [uint32](Get-BigEndianUInt32 $Bytes ($Offset + 16))
            Height = [uint32](Get-BigEndianUInt32 $Bytes ($Offset + 20))
        }
    }

    if ($Length -lt 12) {
        throw (New-Object IO.InvalidDataException("Invalid icon reference."))
    }
    $headerSize = [uint32][BitConverter]::ToUInt32($Bytes, $Offset)
    if ($headerSize -eq 12) {
        $width = [int][BitConverter]::ToUInt16($Bytes, $Offset + 4)
        $storedHeight = [int][BitConverter]::ToUInt16($Bytes, $Offset + 6)
        $planes = [int][BitConverter]::ToUInt16($Bytes, $Offset + 8)
        $bitsPerPixel = [int][BitConverter]::ToUInt16($Bytes, $Offset + 10)
        $compression = 0
        $colorsUsed = 0
        $paletteEntryBytes = 3
    } elseif ($headerSize -in @(40, 52, 56, 108, 124) -and $headerSize -le $Length) {
        $width = [int][BitConverter]::ToInt32($Bytes, $Offset + 4)
        $storedHeight = [int][BitConverter]::ToInt32($Bytes, $Offset + 8)
        $planes = [int][BitConverter]::ToUInt16($Bytes, $Offset + 12)
        $bitsPerPixel = [int][BitConverter]::ToUInt16($Bytes, $Offset + 14)
        $compression = [uint32][BitConverter]::ToUInt32($Bytes, $Offset + 16)
        $colorsUsed = [uint32][BitConverter]::ToUInt32($Bytes, $Offset + 32)
        $paletteEntryBytes = 4
    } else {
        throw (New-Object IO.InvalidDataException("Invalid icon reference."))
    }
    $height = [int]($storedHeight / 2)
    if (
        $width -le 0 -or $width -gt 256 -or
        $height -le 0 -or $height -gt 256 -or ($storedHeight % 2) -ne 0 -or
        $planes -ne 1 -or $bitsPerPixel -notin @(1, 4, 8, 16, 24, 32) -or
        $compression -notin @(0, 3) -or
        ($compression -eq 3 -and $bitsPerPixel -notin @(16, 32)) -or
        $colorsUsed -gt 256 -or
        ($bitsPerPixel -le 8 -and $colorsUsed -gt (1 -shl $bitsPerPixel))
    ) {
        throw (New-Object IO.InvalidDataException("Invalid icon reference."))
    }
    $paletteEntries = if ($colorsUsed -gt 0) {
        [uint64]$colorsUsed
    } elseif ($bitsPerPixel -le 8) {
        [uint64](1 -shl $bitsPerPixel)
    } else {
        [uint64]0
    }
    $externalMaskBytes = if ($compression -eq 3 -and $headerSize -eq 40) { 12 } else { 0 }
    if ($compression -eq 3) {
        $maskOffset = $Offset + 40
        if ($headerSize -eq 40 -and $Length -lt 52) {
            throw (New-Object IO.InvalidDataException("Invalid icon reference."))
        }
        $redMask = [uint32][BitConverter]::ToUInt32($Bytes, $maskOffset)
        $greenMask = [uint32][BitConverter]::ToUInt32($Bytes, $maskOffset + 4)
        $blueMask = [uint32][BitConverter]::ToUInt32($Bytes, $maskOffset + 8)
        $maximumMask = if ($bitsPerPixel -eq 16) { [uint64]0xFFFF } else { [uint64]::MaxValue }
        if (
            $redMask -eq 0 -or $greenMask -eq 0 -or $blueMask -eq 0 -or
            [uint64]$redMask -gt $maximumMask -or
            [uint64]$greenMask -gt $maximumMask -or
            [uint64]$blueMask -gt $maximumMask -or
            -not (Test-ContiguousDibMask $redMask) -or
            -not (Test-ContiguousDibMask $greenMask) -or
            -not (Test-ContiguousDibMask $blueMask) -or
            ($redMask -band $greenMask) -ne 0 -or
            ($redMask -band $blueMask) -ne 0 -or
            ($greenMask -band $blueMask) -ne 0
        ) {
            throw (New-Object IO.InvalidDataException("Invalid icon reference."))
        }
        if ($headerSize -in @(56, 108, 124)) {
            $alphaMask = [uint32][BitConverter]::ToUInt32($Bytes, $maskOffset + 12)
            if (
                $alphaMask -ne 0 -and (
                    [uint64]$alphaMask -gt $maximumMask -or
                    -not (Test-ContiguousDibMask $alphaMask) -or
                    ($alphaMask -band $redMask) -ne 0 -or
                    ($alphaMask -band $greenMask) -ne 0 -or
                    ($alphaMask -band $blueMask) -ne 0
                )
            ) {
                throw (New-Object IO.InvalidDataException("Invalid icon reference."))
            }
        }
    }
    $rowBits = [uint64]$width * [uint64]$bitsPerPixel
    $rowStride = [uint64][Math]::Ceiling([double]$rowBits / 32.0) * 4
    $andStride = [uint64][Math]::Ceiling([double]$width / 32.0) * 4
    $minimumLength = [uint64]$headerSize +
        ($paletteEntries * [uint64]$paletteEntryBytes) + [uint64]$externalMaskBytes +
        ($rowStride * [uint64]$height) + ($andStride * [uint64]$height)
    if ($minimumLength -gt [uint64]$Length) {
        throw (New-Object IO.InvalidDataException("Invalid icon reference."))
    }
    return [pscustomobject]@{
        Width = [uint32]$width
        Height = [uint32]$height
    }
}

function Get-WindowsIcoFrameSizes([IO.FileInfo]$File) {
    if ($null -eq $File -or $File.Extension -ine ".ico") {
        throw (New-Object IO.InvalidDataException("Invalid icon reference."))
    }
    $bytes = [byte[]](Read-BoundedPackageBytes $File.FullName 8388608)
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
        $entryOffset = 6 + (16 * $index)
        $width = [int]$bytes[$entryOffset]
        $height = [int]$bytes[$entryOffset + 1]
        if ($width -eq 0) { $width = 256 }
        if ($height -eq 0) { $height = 256 }
        $imageBytes = [uint64][BitConverter]::ToUInt32($bytes, $entryOffset + 8)
        $imageOffset = [uint64][BitConverter]::ToUInt32($bytes, $entryOffset + 12)
        $imageEnd = $imageOffset + $imageBytes
        if (
            $width -ne $height -or $imageBytes -eq 0 -or
            $imageOffset -lt $directoryEnd -or $imageEnd -gt $bytes.Length
        ) {
            throw (New-Object IO.InvalidDataException("Invalid icon reference."))
        }
        $dimensions = Get-IcoPayloadDimensions $bytes ([int]$imageOffset) ([int]$imageBytes)
        if ($dimensions.Width -ne $width -or $dimensions.Height -ne $height) {
            throw (New-Object IO.InvalidDataException("Invalid icon reference."))
        }
        $sizes[$index] = $width
    }
    return $sizes
}

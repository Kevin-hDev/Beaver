$ErrorActionPreference = "Stop"

function Assert-IconTrue([bool]$Value) {
    if (-not $Value) { throw "Windows icon validation contract failed." }
}

function Assert-IconFalse([bool]$Value) {
    if ($Value) { throw "Windows icon validation contract failed." }
}

. (Join-Path $PSScriptRoot "windows-artifact-helpers.ps1")
. (Join-Path $PSScriptRoot "windows-icon-validation.test-fixtures.ps1")

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
    $referenceIconPath = [IO.Path]::GetFullPath(
        (Join-Path $PSScriptRoot "../../src-tauri/icons/icon.ico")
    )
    $referenceIconFile = Get-BoundedPackageFile $referenceIconPath 8388608
    Assert-IconTrue (Test-WindowsReferenceIcon $referenceIconFile)

    $validDib = New-TestSolidDibFrame 16
    $dibDimensions = Get-IcoPayloadDimensions $validDib 0 $validDib.Length
    Assert-IconTrue ($dibDimensions.Width -eq 16 -and $dibDimensions.Height -eq 16)
    $validBitfields = New-Object byte[] ($validDib.Length + 12)
    [Array]::Copy($validDib, 0, $validBitfields, 0, 40)
    [BitConverter]::GetBytes([uint32]3).CopyTo($validBitfields, 16)
    [BitConverter]::GetBytes([uint32]0x00FF0000).CopyTo($validBitfields, 40)
    [BitConverter]::GetBytes([uint32]0x0000FF00).CopyTo($validBitfields, 44)
    [BitConverter]::GetBytes([uint32]0x000000FF).CopyTo($validBitfields, 48)
    [Array]::Copy($validDib, 40, $validBitfields, 52, $validDib.Length - 40)
    $bitfieldsDimensions = Get-IcoPayloadDimensions $validBitfields 0 $validBitfields.Length
    Assert-IconTrue ($bitfieldsDimensions.Width -eq 16 -and $bitfieldsDimensions.Height -eq 16)
    $valid16Bitfields = New-Object byte[] (40 + 12 + 512 + 64)
    [BitConverter]::GetBytes([uint32]40).CopyTo($valid16Bitfields, 0)
    [BitConverter]::GetBytes([int32]16).CopyTo($valid16Bitfields, 4)
    [BitConverter]::GetBytes([int32]32).CopyTo($valid16Bitfields, 8)
    [BitConverter]::GetBytes([uint16]1).CopyTo($valid16Bitfields, 12)
    [BitConverter]::GetBytes([uint16]16).CopyTo($valid16Bitfields, 14)
    [BitConverter]::GetBytes([uint32]3).CopyTo($valid16Bitfields, 16)
    [BitConverter]::GetBytes([uint32]0xF800).CopyTo($valid16Bitfields, 40)
    [BitConverter]::GetBytes([uint32]0x07E0).CopyTo($valid16Bitfields, 44)
    [BitConverter]::GetBytes([uint32]0x001F).CopyTo($valid16Bitfields, 48)
    $bitfields16Dimensions = Get-IcoPayloadDimensions $valid16Bitfields 0 $valid16Bitfields.Length
    Assert-IconTrue ($bitfields16Dimensions.Width -eq 16 -and $bitfields16Dimensions.Height -eq 16)
    $validAlphaBitfields = New-Object byte[] (108 + 512 + 64)
    [BitConverter]::GetBytes([uint32]108).CopyTo($validAlphaBitfields, 0)
    [BitConverter]::GetBytes([int32]16).CopyTo($validAlphaBitfields, 4)
    [BitConverter]::GetBytes([int32]32).CopyTo($validAlphaBitfields, 8)
    [BitConverter]::GetBytes([uint16]1).CopyTo($validAlphaBitfields, 12)
    [BitConverter]::GetBytes([uint16]16).CopyTo($validAlphaBitfields, 14)
    [BitConverter]::GetBytes([uint32]3).CopyTo($validAlphaBitfields, 16)
    [BitConverter]::GetBytes([uint32]0x7C00).CopyTo($validAlphaBitfields, 40)
    [BitConverter]::GetBytes([uint32]0x03E0).CopyTo($validAlphaBitfields, 44)
    [BitConverter]::GetBytes([uint32]0x001F).CopyTo($validAlphaBitfields, 48)
    [BitConverter]::GetBytes([uint32]0x8000).CopyTo($validAlphaBitfields, 52)
    $alphaDimensions = Get-IcoPayloadDimensions `
        $validAlphaBitfields 0 $validAlphaBitfields.Length
    Assert-IconTrue ($alphaDimensions.Width -eq 16 -and $alphaDimensions.Height -eq 16)

    $validIndexed = New-Object byte[] (40 + 8 + 64 + 64)
    [BitConverter]::GetBytes([uint32]40).CopyTo($validIndexed, 0)
    [BitConverter]::GetBytes([int32]16).CopyTo($validIndexed, 4)
    [BitConverter]::GetBytes([int32]32).CopyTo($validIndexed, 8)
    [BitConverter]::GetBytes([uint16]1).CopyTo($validIndexed, 12)
    [BitConverter]::GetBytes([uint16]1).CopyTo($validIndexed, 14)
    [BitConverter]::GetBytes([uint32]2).CopyTo($validIndexed, 32)
    $indexedDimensions = Get-IcoPayloadDimensions $validIndexed 0 $validIndexed.Length
    Assert-IconTrue ($indexedDimensions.Width -eq 16 -and $indexedDimensions.Height -eq 16)

    $truncatedDibRejected = $false
    try {
        [void](Get-IcoPayloadDimensions $validDib 0 40)
    } catch [IO.InvalidDataException] {
        $truncatedDibRejected = $true
    }
    Assert-IconTrue $truncatedDibRejected

    $invalidDibFrames = @()
    $invalidHeader = [byte[]]$validDib.Clone()
    [BitConverter]::GetBytes([uint32]13).CopyTo($invalidHeader, 0)
    $invalidDibFrames += ,$invalidHeader
    $invalidWidth = [byte[]]$validDib.Clone()
    [BitConverter]::GetBytes([int32]0).CopyTo($invalidWidth, 4)
    $invalidDibFrames += ,$invalidWidth
    $invalidHeight = [byte[]]$validDib.Clone()
    [BitConverter]::GetBytes([int32]31).CopyTo($invalidHeight, 8)
    $invalidDibFrames += ,$invalidHeight
    $unsupportedHeader = [byte[]]$validDib.Clone()
    [BitConverter]::GetBytes([uint32]41).CopyTo($unsupportedHeader, 0)
    $invalidDibFrames += ,$unsupportedHeader
    $xorOnlyLength = 40 + (16 * 16 * 4)
    $xorOnly = New-Object byte[] $xorOnlyLength
    [Array]::Copy($validDib, $xorOnly, $xorOnlyLength)
    $invalidDibFrames += ,$xorOnly
    $bitfieldsWithoutMasks = [byte[]]$validDib.Clone()
    [BitConverter]::GetBytes([uint32]3).CopyTo($bitfieldsWithoutMasks, 16)
    $invalidDibFrames += ,$bitfieldsWithoutMasks
    $indexedWithoutPalette = New-Object byte[] (40 + 64 + 64)
    [BitConverter]::GetBytes([uint32]40).CopyTo($indexedWithoutPalette, 0)
    [BitConverter]::GetBytes([int32]16).CopyTo($indexedWithoutPalette, 4)
    [BitConverter]::GetBytes([int32]32).CopyTo($indexedWithoutPalette, 8)
    [BitConverter]::GetBytes([uint16]1).CopyTo($indexedWithoutPalette, 12)
    [BitConverter]::GetBytes([uint16]1).CopyTo($indexedWithoutPalette, 14)
    $invalidDibFrames += ,$indexedWithoutPalette
    $oversizedPalette = [byte[]]$validIndexed.Clone()
    [BitConverter]::GetBytes([uint32]3).CopyTo($oversizedPalette, 32)
    $invalidDibFrames += ,$oversizedPalette
    $outOfRangeMasks = [byte[]]$validBitfields.Clone()
    [BitConverter]::GetBytes([uint16]16).CopyTo($outOfRangeMasks, 14)
    $invalidDibFrames += ,$outOfRangeMasks
    $nonContiguousMasks = [byte[]]$validBitfields.Clone()
    [BitConverter]::GetBytes([uint16]16).CopyTo($nonContiguousMasks, 14)
    [BitConverter]::GetBytes([uint32]0x00005555).CopyTo($nonContiguousMasks, 40)
    [BitConverter]::GetBytes([uint32]0x00002AAA).CopyTo($nonContiguousMasks, 44)
    [BitConverter]::GetBytes([uint32]0x00008000).CopyTo($nonContiguousMasks, 48)
    $invalidDibFrames += ,$nonContiguousMasks
    $overlappingAlpha = [byte[]]$validAlphaBitfields.Clone()
    [BitConverter]::GetBytes([uint32]0x7C00).CopyTo($overlappingAlpha, 52)
    $invalidDibFrames += ,$overlappingAlpha
    foreach ($invalidDib in $invalidDibFrames) {
        $invalidDibRejected = $false
        try {
            [void](Get-IcoPayloadDimensions $invalidDib 0 $invalidDib.Length)
        } catch [IO.InvalidDataException] {
            $invalidDibRejected = $true
        }
        Assert-IconTrue $invalidDibRejected
    }

    $missingFramePath = Join-Path $temporaryRoot "missing-32.ico"
    Copy-TestIcoWithoutFrame $referenceIconPath $missingFramePath 32
    $missingFrameFile = Get-BoundedPackageFile $missingFramePath 8388608
    $missingSizes = @(Get-WindowsIcoFrameSizes $missingFrameFile)
    Assert-IconFalse (32 -in $missingSizes)
    Assert-IconFalse (Test-WindowsReferenceIcon $missingFrameFile)

    $aliasedFramePath = Join-Path $temporaryRoot "48-points-to-32.ico"
    Copy-TestIcoWithAliasedFrame $referenceIconPath $aliasedFramePath 48 32
    $aliasedFrameFile = Get-BoundedPackageFile $aliasedFramePath 8388608
    Assert-IconFalse (Test-WindowsReferenceIcon $aliasedFrameFile)

    foreach ($fieldOffset in @(8, 12)) {
        $invalidBoundsPath = Join-Path $temporaryRoot "invalid-bounds-$fieldOffset.ico"
        $invalidBounds = [IO.File]::ReadAllBytes($referenceIconPath)
        [BitConverter]::GetBytes([uint32]::MaxValue).CopyTo(
            $invalidBounds,
            6 + $fieldOffset
        )
        [IO.File]::WriteAllBytes($invalidBoundsPath, $invalidBounds)
        $invalidBoundsFile = Get-BoundedPackageFile $invalidBoundsPath 8388608
        Assert-IconFalse (Test-WindowsReferenceIcon $invalidBoundsFile)
    }

    $fixtureSource = Join-Path $temporaryRoot "icon-fixtures.cs"
    $brandedExecutable = Join-Path $temporaryRoot "branded.exe"
    New-TestIconExecutable $referenceIconPath $fixtureSource $brandedExecutable
    $brandedFile = Get-BoundedPackageFile $brandedExecutable 67108864
    Assert-IconTrue ($null -eq (Get-NativeIconContentFailure $brandedFile $referenceIconFile))

    foreach ($size in @(16, 32, 48, 256)) {
        $changedIconPath = Join-Path $temporaryRoot "changed-$size.ico"
        Copy-TestIcoWithSolidFrame $referenceIconPath $changedIconPath $size
        $changedIconReference = Get-BoundedPackageFile $changedIconPath 8388608
        Assert-IconTrue (Test-WindowsReferenceIcon $changedIconReference)
        $changedExecutable = Join-Path $temporaryRoot "changed-$size.exe"
        New-TestIconExecutable $changedIconPath $fixtureSource $changedExecutable
        $changedFile = Get-BoundedPackageFile $changedExecutable 67108864
        Assert-IconTrue (
            (Get-NativeIconContentFailure $changedFile $referenceIconFile) -ceq "content"
        )
    }

} finally {
    if (Test-Path -LiteralPath $temporaryRoot) {
        if (
            [IO.Path]::GetDirectoryName($temporaryRoot) -cne $temporaryBase -or
            [IO.Path]::GetFileName($temporaryRoot) -notmatch "^[a-f0-9]{32}$"
        ) {
            throw "Windows icon validation contract failed."
        }
        Remove-Item -LiteralPath $temporaryRoot -Recurse -Force
    }
}

Write-Host "Windows icon validation contracts OK"

function Get-TestIcoEntries([byte[]]$Bytes) {
    $count = [int][BitConverter]::ToUInt16($Bytes, 4)
    $entries = @()
    for ($index = 0; $index -lt $count; $index += 1) {
        $directoryOffset = 6 + (16 * $index)
        $size = [int]$Bytes[$directoryOffset]
        if ($size -eq 0) { $size = 256 }
        $entries += [pscustomobject]@{
            DirectoryOffset = $directoryOffset
            Size = $size
            PayloadBytes = [int][BitConverter]::ToUInt32($Bytes, $directoryOffset + 8)
            PayloadOffset = [int][BitConverter]::ToUInt32($Bytes, $directoryOffset + 12)
        }
    }
    return $entries
}

function Copy-TestIcoWithoutFrame([string]$Source, [string]$Destination, [int]$Size) {
    $sourceBytes = [IO.File]::ReadAllBytes($Source)
    $entries = @(Get-TestIcoEntries $sourceBytes | Where-Object { $_.Size -ne $Size })
    $directoryEnd = 6 + (16 * $entries.Count)
    $payloadLength = ($entries | Measure-Object -Property PayloadBytes -Sum).Sum
    $result = New-Object byte[] ($directoryEnd + $payloadLength)
    [Array]::Copy($sourceBytes, 0, $result, 0, 4)
    [BitConverter]::GetBytes([uint16]$entries.Count).CopyTo($result, 4)
    $nextPayloadOffset = $directoryEnd
    for ($index = 0; $index -lt $entries.Count; $index += 1) {
        $entry = $entries[$index]
        $newDirectoryOffset = 6 + (16 * $index)
        [Array]::Copy($sourceBytes, $entry.DirectoryOffset, $result, $newDirectoryOffset, 16)
        [BitConverter]::GetBytes([uint32]$nextPayloadOffset).CopyTo($result, $newDirectoryOffset + 12)
        [Array]::Copy(
            $sourceBytes,
            $entry.PayloadOffset,
            $result,
            $nextPayloadOffset,
            $entry.PayloadBytes
        )
        $nextPayloadOffset += $entry.PayloadBytes
    }
    [IO.File]::WriteAllBytes($Destination, $result)
}

function Copy-TestIcoWithAliasedFrame(
    [string]$Source,
    [string]$Destination,
    [int]$DeclaredSize,
    [int]$PayloadSize
) {
    $bytes = [IO.File]::ReadAllBytes($Source)
    $entries = @(Get-TestIcoEntries $bytes)
    $declared = $entries | Where-Object { $_.Size -eq $DeclaredSize } | Select-Object -First 1
    $payload = $entries | Where-Object { $_.Size -eq $PayloadSize } | Select-Object -First 1
    [BitConverter]::GetBytes([uint32]$payload.PayloadBytes).CopyTo(
        $bytes,
        $declared.DirectoryOffset + 8
    )
    [BitConverter]::GetBytes([uint32]$payload.PayloadOffset).CopyTo(
        $bytes,
        $declared.DirectoryOffset + 12
    )
    [IO.File]::WriteAllBytes($Destination, $bytes)
}

function New-TestSolidDibFrame([int]$Size) {
    $pixelBytes = $Size * $Size * 4
    $maskStride = [int](($Size + 31) / 32) * 4
    $bytes = New-Object byte[] (40 + $pixelBytes + ($maskStride * $Size))
    [BitConverter]::GetBytes([uint32]40).CopyTo($bytes, 0)
    [BitConverter]::GetBytes([int32]$Size).CopyTo($bytes, 4)
    [BitConverter]::GetBytes([int32]($Size * 2)).CopyTo($bytes, 8)
    [BitConverter]::GetBytes([uint16]1).CopyTo($bytes, 12)
    [BitConverter]::GetBytes([uint16]32).CopyTo($bytes, 14)
    [BitConverter]::GetBytes([uint32]$pixelBytes).CopyTo($bytes, 20)
    for ($offset = 40; $offset -lt (40 + $pixelBytes); $offset += 4) {
        $bytes[$offset] = 255
        $bytes[$offset + 2] = 255
        $bytes[$offset + 3] = 255
    }
    return $bytes
}

function Copy-TestIcoWithSolidFrame(
    [string]$Source,
    [string]$Destination,
    [int]$Size
) {
    $sourceBytes = [IO.File]::ReadAllBytes($Source)
    $entry = @(Get-TestIcoEntries $sourceBytes | Where-Object { $_.Size -eq $Size })[0]
    $payload = New-TestSolidDibFrame $Size
    $result = New-Object byte[] ($sourceBytes.Length + $payload.Length)
    [Array]::Copy($sourceBytes, $result, $sourceBytes.Length)
    [Array]::Copy($payload, 0, $result, $sourceBytes.Length, $payload.Length)
    [BitConverter]::GetBytes([uint32]$payload.Length).CopyTo(
        $result,
        $entry.DirectoryOffset + 8
    )
    [BitConverter]::GetBytes([uint32]$sourceBytes.Length).CopyTo(
        $result,
        $entry.DirectoryOffset + 12
    )
    [IO.File]::WriteAllBytes($Destination, $result)
}

function New-TestIconExecutable([string]$Icon, [string]$Source, [string]$Output) {
    $compiler = @(
        (Join-Path $env:WINDIR "Microsoft.NET\Framework64\v4.0.30319\csc.exe"),
        (Join-Path $env:WINDIR "Microsoft.NET\Framework\v4.0.30319\csc.exe")
    ).Where({ Test-Path -LiteralPath $_ -PathType Leaf }, "First")
    if ($compiler.Count -ne 1) { throw "PowerShell package contract failed." }
    [IO.File]::WriteAllText(
        $Source,
        "public static class BeaverIconFixture { public static void Main() {} }"
    )
    & $compiler[0] /nologo /target:winexe "/win32icon:$Icon" "/out:$Output" $Source
    if ($LASTEXITCODE -ne 0) { throw "PowerShell package contract failed." }
}
